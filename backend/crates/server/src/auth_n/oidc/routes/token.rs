use crate::{
    auth_n::{
        certificates::encoding_key,
        claims::UserTokenClaims,
        oidc::{
            extract::client_app::find_client_app,
            schemas::{self, token_body::OAuth2TokenBody},
        },
    },
    extract::path_tenant::{Project, Tenant},
    services::AppState,
};
use axum::{
    Json,
    extract::State,
    response::{IntoResponse, Response},
};
use axum_extra::routing::TypedPath;
use base64::{Engine as _, engine::general_purpose::URL_SAFE};
use jsonwebtoken::{Algorithm, Header};
use oxidized_fhir_model::r4::generated::terminology::IssueType;
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_repository::{
    Repository,
    admin::ProjectAuthAdmin,
    types::{
        AuthorId, AuthorKind,
        authorization_code::{
            AuthorizationCode, AuthorizationCodeSearchClaims, CreateAuthorizationCode,
            PKCECodeChallengeMethod,
        },
        user::UserRole,
    },
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;

#[derive(TypedPath)]
#[typed_path("/token")]
pub struct TokenPath;

#[derive(Serialize, Deserialize, Debug)]
pub enum TokenType {
    Bearer,
}

pub static TOKEN_EXPIRATION: usize = 3600; // 1 hour

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenResponse {
    access_token: String,
    id_token: String,
    token_type: TokenType,
    expires_in: usize,
}

pub fn verify_code_verifier(
    code: &AuthorizationCode,
    code_verifier: &str,
) -> Result<(), OperationOutcomeError> {
    match code.pkce_code_challenge_method {
        Some(PKCECodeChallengeMethod::S256) => {
            let mut hasher = Sha256::new();
            hasher.update(code_verifier.as_bytes());
            let hashed = hasher.finalize();

            let mut computed_challenge = URL_SAFE.encode(&hashed);
            // Remove last character which is an equal.
            computed_challenge.pop();

            if Some(computed_challenge.as_str())
                != code.pkce_code_challenge.as_ref().map(|v| v.as_str())
            {
                return Err(OperationOutcomeError::error(
                    IssueType::Invalid(None),
                    "PKCE code verifier does not match the code challenge.".to_string(),
                ));
            }

            Ok(())
        }
        Some(PKCECodeChallengeMethod::Plain) => {
            if code_verifier != code.pkce_code_challenge.as_deref().unwrap_or("") {
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

pub async fn token<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: TokenPath,
    Tenant { tenant }: Tenant,
    Project { project }: Project,
    State(state): State<Arc<AppState<Repo, Search, Terminology>>>,
    Json(token_body): Json<schemas::token_body::OAuth2TokenBody>,
) -> Result<Response, OperationOutcomeError> {
    match token_body {
        OAuth2TokenBody::AuthorizationCode {
            client_id,
            client_secret,
            code,
            code_verifier,
            redirect_uri,
        } => {
            let client_app =
                find_client_app(&state, tenant.clone(), project.clone(), client_id.clone()).await?;

            if client_secret != client_app.secret.and_then(|v| v.value) {
                return Err(OperationOutcomeError::error(
                    IssueType::Security(None),
                    "Invalid client secret".to_string(),
                ));
            }

            let code: Vec<AuthorizationCode> = ProjectAuthAdmin::search(
                &*state.repo,
                &tenant,
                &project,
                &AuthorizationCodeSearchClaims {
                    client_id: Some(client_id.clone()),
                    code: Some(code),
                    user_id: None,
                },
            )
            .await?;

            if let Some(code) = code.get(0) {
                if code.is_expired.unwrap_or(false) {
                    return Err(OperationOutcomeError::fatal(
                        IssueType::Security(None),
                        "Authorization code has expired.".to_string(),
                    ));
                }

                if let Err(_e) = verify_code_verifier(&code, &code_verifier) {
                    return Err(OperationOutcomeError::fatal(
                        IssueType::Invalid(None),
                        "Failed to verify PKCE code verifier.".to_string(),
                    ));
                }

                if code.redirect_uri != Some(redirect_uri) {
                    return Err(OperationOutcomeError::fatal(
                        IssueType::Invalid(None),
                        "Redirect URI does not match the one used to create the authorization code.".to_string(),
                    ));
                }

                // Remove the code once valid.
                ProjectAuthAdmin::<CreateAuthorizationCode, _, _, _, _>::delete(
                    &*state.repo,
                    &tenant,
                    &project,
                    &code.code,
                )
                .await?;

                let token = jsonwebtoken::encode(
                    &Header::new(Algorithm::RS256),
                    &UserTokenClaims {
                        sub: AuthorId::new(code.user_id.clone()),
                        exp: (chrono::Utc::now()
                            + chrono::Duration::seconds(TOKEN_EXPIRATION as i64))
                        .timestamp() as usize,
                        aud: client_id,
                        scope: "".to_string(),
                        tenant: tenant,
                        project: Some(project),
                        user_role: UserRole::Member,
                        user_id: AuthorId::new(code.user_id.clone()),
                        resource_type: AuthorKind::Membership,
                        access_policy_version_ids: vec![],
                    },
                    encoding_key(),
                )
                .map_err(|_| {
                    OperationOutcomeError::error(
                        IssueType::Exception(None),
                        "Failed to create access token.".to_string(),
                    )
                })?;

                Ok(Json(TokenResponse {
                    access_token: token.clone(),
                    id_token: token,
                    expires_in: TOKEN_EXPIRATION,
                    token_type: TokenType::Bearer,
                })
                .into_response())
            } else {
                Err(OperationOutcomeError::fatal(
                    IssueType::Invalid(None),
                    "The provided authorization code is invalid.".to_string(),
                ))
            }
        }

        _ => Err(OperationOutcomeError::fatal(
            IssueType::NotSupported(None),
            "The provided grant type is not supported.".to_string(),
        )),
    }
}
