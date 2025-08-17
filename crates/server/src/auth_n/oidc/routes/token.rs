use crate::{
    AppState,
    auth_n::{
        certificates::encoding_key,
        oidc::{
            extract::client_app::find_client_app,
            schemas::{self, token_body::OAuth2TokenBody},
        },
    },
    extract::path_tenant::TenantProject,
};
use axum::{
    Json,
    extract::State,
    response::{IntoResponse, Response},
};
use axum_extra::routing::TypedPath;
use base64::{Engine as _, engine::general_purpose::URL_SAFE};
use jsonwebtoken::{Algorithm, Header};
use oxidized_fhir_operation_error::{OperationOutcomeCodes, OperationOutcomeError};
use oxidized_fhir_search::SearchEngine;
use oxidized_repository::{
    Repository,
    admin::ProjectAuthAdmin,
    types::{
        ResourceId, TenantId, VersionId,
        authorization_code::{
            AuthorizationCode, AuthorizationCodeSearchClaims, PKCECodeChallengeMethod,
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
pub struct TokenResponse {
    token: String,
}

#[derive(Serialize, Deserialize, Debug)]
enum UserResourceTypes {
    Membership,
    ClientApplication,
    OperationDefinition,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenClaims {
    sub: ResourceId,
    exp: usize,
    aud: String,
    scope: String,

    #[serde(rename = "https://oxidized-health.app/tenant")]
    tenant: TenantId,
    #[serde(rename = "https://oxidized-health.app/user_role")]
    user_role: UserRole,
    #[serde(rename = "https://oxidized-health.app/user_id")]
    user_id: Option<ResourceId>,
    #[serde(rename = "https://oxidized-health.app/resource_type")]
    resource_type: UserResourceTypes,
    #[serde(rename = "https://oxidized-health.app/access_policies")]
    access_policy_version_ids: Vec<VersionId>,
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
                    OperationOutcomeCodes::Invalid,
                    "PKCE code verifier does not match the code challenge.".to_string(),
                ));
            }

            Ok(())
        }
        Some(PKCECodeChallengeMethod::Plain) => {
            if code_verifier != code.pkce_code_challenge.as_deref().unwrap_or("") {
                return Err(OperationOutcomeError::error(
                    OperationOutcomeCodes::Invalid,
                    "PKCE code verifier does not match the code challenge.".to_string(),
                ));
            }
            Ok(())
        }
        _ => Err(OperationOutcomeError::error(
            OperationOutcomeCodes::Invalid,
            "PKCE code challenge method not supported.".to_string(),
        )),
    }
}

pub async fn token<Repo: Repository + Send + Sync, Search: SearchEngine + Send + Sync>(
    _: TokenPath,
    tenant: TenantProject,
    State(state): State<Arc<AppState<Repo, Search>>>,
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
            let client_app = find_client_app(
                &state,
                tenant.tenant.clone(),
                tenant.project.clone(),
                client_id.clone(),
            )
            .await?;

            if client_secret != client_app.secret.and_then(|v| v.value) {
                return Err(OperationOutcomeError::error(
                    OperationOutcomeCodes::Security,
                    "Invalid client secret".to_string(),
                ));
            }

            let code: Vec<AuthorizationCode> = ProjectAuthAdmin::search(
                &state.repo,
                &tenant.tenant,
                &tenant.project,
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
                        OperationOutcomeCodes::Security,
                        "Authorization code has expired.".to_string(),
                    ));
                }

                if let Err(_e) = verify_code_verifier(&code, &code_verifier) {
                    return Err(OperationOutcomeError::fatal(
                        OperationOutcomeCodes::Invalid,
                        "Failed to verify PKCE code verifier.".to_string(),
                    ));
                }

                if code.redirect_uri != Some(redirect_uri) {
                    return Err(OperationOutcomeError::fatal(
                        OperationOutcomeCodes::Invalid,
                        "Redirect URI does not match the one used to create the authorization code.".to_string(),
                    ));
                }

                // Remove the code once valid.
                ProjectAuthAdmin::delete(&state.repo, &tenant.tenant, &tenant.project, &code.code)
                    .await?;

                let token = jsonwebtoken::encode(
                    &Header::new(Algorithm::RS256),
                    &TokenClaims {
                        sub: ResourceId::new(code.user_id.clone()),
                        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
                        aud: client_id,
                        scope: "".to_string(),
                        tenant: tenant.tenant,
                        user_role: UserRole::Member,
                        user_id: Some(ResourceId::new(code.user_id.clone())),
                        resource_type: UserResourceTypes::Membership,
                        access_policy_version_ids: vec![],
                    },
                    encoding_key(),
                )
                .map_err(|_| {
                    OperationOutcomeError::error(
                        OperationOutcomeCodes::Exception,
                        "Failed to create access token.".to_string(),
                    )
                })?;

                Ok(Json(TokenResponse { token }).into_response())
            } else {
                Err(OperationOutcomeError::fatal(
                    OperationOutcomeCodes::Invalid,
                    "The provided authorization code is invalid.".to_string(),
                ))
            }
        }

        _ => Err(OperationOutcomeError::fatal(
            OperationOutcomeCodes::NotSupported,
            "The provided grant type is not supported.".to_string(),
        )),
    }
}
