use base64::{Engine as _, engine::general_purpose::URL_SAFE};
use oxidized_fhir_model::r4::generated::{resources::ClientApplication, terminology::IssueType};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_jwt::{ProjectId, TenantId};
use oxidized_repository::{
    Repository,
    admin::ProjectAuthAdmin,
    types::authorization_code::{
        AuthorizationCode, AuthorizationCodeSearchClaims, PKCECodeChallengeMethod,
    },
    utilities::generate_id,
};
use sha2::{Digest, Sha256};

pub fn generate_code_verifier() -> String {
    // Generate a random code verifier between 43 and 128 characters.
    let code_verifier = generate_id(Some(100));
    code_verifier
}

pub fn generate_code_challenge(
    code_verifier: &str,
    method: &PKCECodeChallengeMethod,
) -> Result<String, OperationOutcomeError> {
    match method {
        PKCECodeChallengeMethod::S256 => {
            let mut hasher = Sha256::new();
            hasher.update(code_verifier.as_bytes());
            let hashed = hasher.finalize();

            let mut computed_challenge = URL_SAFE.encode(&hashed);
            // Remove last character which is an equal.
            computed_challenge.pop();

            Ok(computed_challenge)
        }
        PKCECodeChallengeMethod::Plain => Ok(code_verifier.to_string()),
    }
}

pub fn verify_code_verifier(
    pkce_code_challenge: &Option<String>,
    pkce_code_challenge_method: &Option<PKCECodeChallengeMethod>,
    code_verifier: &str,
) -> Result<(), OperationOutcomeError> {
    match pkce_code_challenge_method {
        Some(method) => {
            let computed_challenge = generate_code_challenge(code_verifier, method)?;

            if Some(computed_challenge) != *pkce_code_challenge {
                return Err(OperationOutcomeError::error(
                    IssueType::Invalid(None),
                    "PKCE code verifier does not match the code challenge.".to_string(),
                ));
            }

            Ok(())
        }

        _ => Err(OperationOutcomeError::error(
            IssueType::Invalid(None),
            "PKCE code challenge method not supported.".to_string(),
        )),
    }
}

pub async fn retrieve_and_verify_code<Repo: Repository>(
    repo: &Repo,
    tenant: &TenantId,
    project: &ProjectId,
    client: &ClientApplication,
    code: &str,
    redirect_uri: Option<&str>,
    code_verifier: Option<&str>,
) -> Result<AuthorizationCode, OperationOutcomeError> {
    let mut code: Vec<AuthorizationCode> = ProjectAuthAdmin::search(
        repo,
        &tenant,
        &project,
        &AuthorizationCodeSearchClaims {
            client_id: client.id.clone(),
            code: Some(code.to_string()),
            user_id: None,
        },
    )
    .await?;

    if let Some(code) = code.pop() {
        if code.project.as_ref() != Some(project) {
            return Err(OperationOutcomeError::fatal(
                IssueType::Invalid(None),
                "Authorization code does not belong to the specified project.".to_string(),
            ));
        }

        if code.tenant != *tenant {
            return Err(OperationOutcomeError::fatal(
                IssueType::Invalid(None),
                "Authorization code does not belong to the specified tenant.".to_string(),
            ));
        }

        if code.is_expired.unwrap_or(true) {
            return Err(OperationOutcomeError::fatal(
                IssueType::Security(None),
                "Authorization code has expired.".to_string(),
            ));
        }

        if let Some(code_verifier) = code_verifier
            && verify_code_verifier(
                &code.pkce_code_challenge,
                &code.pkce_code_challenge_method,
                &code_verifier,
            )
            .is_err()
        {
            return Err(OperationOutcomeError::fatal(
                IssueType::Invalid(None),
                "Failed to verify PKCE code verifier.".to_string(),
            ));
        }

        if code.client_id.as_ref().map(|c| c.as_str()) != client.id.as_ref().map(|c| c.as_str()) {
            return Err(OperationOutcomeError::fatal(
                IssueType::Invalid(None),
                "Invalid authorization code.".to_string(),
            ));
        }

        if code.redirect_uri.as_ref().map(String::as_str) != redirect_uri {
            return Err(OperationOutcomeError::fatal(
                IssueType::Invalid(None),
                "Redirect URI does not match the one used to create the authorization code."
                    .to_string(),
            ));
        }

        Ok(code)
    } else {
        return Err(OperationOutcomeError::fatal(
            IssueType::Invalid(None),
            "Authorization code not found.".to_string(),
        ));
    }
}
