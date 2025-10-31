use oxidized_fhir_model::r4::generated::{resources::ClientApplication, terminology::IssueType};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_jwt::TenantId;
use oxidized_repository::{
    Repository,
    admin::TenantAuthAdmin,
    types::user::{CreateUser, UpdateUser},
};
use regex::Regex;

pub fn is_valid_redirect_url(redirect_url: &str, client: &ClientApplication) -> bool {
    let k = client.redirectUri.as_ref().and_then(|redirect_uris| {
        redirect_uris.iter().find(|redirect_pattern| {
            if let Some(redirect_pattern) = redirect_pattern.value.as_ref()
                && let Ok(pattern) = Regex::new(&redirect_pattern.replace("*", "(.+)"))
            {
                pattern.is_match(redirect_url)
            } else {
                false
            }
        })
    });

    k.is_some() && !redirect_url.is_empty()
}

pub async fn set_user_password<Repo: Repository>(
    repo: &Repo,
    tenant: &TenantId,
    user_email: &str,
    user_id: &str,
    password: &str,
) -> Result<(), OperationOutcomeError> {
    // In a real implementation, you would hash the password here
    let password_strength = zxcvbn::zxcvbn(password, &[user_email]);

    if u8::from(password_strength.score()) < 3 {
        let feedback = password_strength
            .feedback()
            .map(|f| format!("{}", f))
            .unwrap_or_default();

        return Err(OperationOutcomeError::fatal(
            IssueType::Security(None),
            feedback,
        ));
    }

    TenantAuthAdmin::<CreateUser, _, _, _, String>::update(
        repo,
        &tenant,
        UpdateUser {
            id: user_id.to_string(),
            password: Some(password.to_string()),
            email: None,
            role: None,
            method: None,
            provider_id: None,
        },
    )
    .await?;

    Ok(())
}
