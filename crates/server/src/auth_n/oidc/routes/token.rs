use crate::{
    AppState,
    auth_n::{
        certificates::encoding_key,
        oidc::{
            middleware::OIDCParameters,
            schemas::{self, token_body::OAuth2TokenBody},
        },
        session,
    },
    extract::path_tenant::TenantProject,
};
use axum::{
    Extension, Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_extra::routing::TypedPath;
use jsonwebtoken::{Algorithm, Header};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::SearchEngine;
use oxidized_repository::{
    Repository,
    admin::ProjectAuthAdmin,
    types::authorization_code::{AuthorizationCode, AuthorizationCodeSearchClaims},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_sessions::Session;

#[derive(TypedPath)]
#[typed_path("/token")]
pub struct TokenPath;

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenResponse {
    token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenClaims {
    sub: String,
    exp: usize,
}

pub async fn token<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
>(
    _: TokenPath,
    tenant: TenantProject,
    State(state): State<Arc<AppState<Repo, Search>>>,
    current_session: Session,
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
            let code: Vec<AuthorizationCode> = ProjectAuthAdmin::search(
                &state.repo,
                &tenant.tenant,
                &tenant.project,
                &AuthorizationCodeSearchClaims {
                    client_id: Some(client_id),
                    code: Some(code),
                    user_id: None,
                },
            )
            .await?;

            if let Some(code) = code.get(0) {
                if code.is_expired.unwrap_or(false) {
                    return Err(OperationOutcomeError::fatal(
                        "invalid".to_string(),
                        "Authorization code has expired.".to_string(),
                    ));
                }

                let token = jsonwebtoken::encode(
                    &Header::new(Algorithm::RS256),
                    &TokenClaims {
                        sub: "random".to_string(),
                        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
                    },
                    encoding_key(),
                )
                .map_err(|_| {
                    OperationOutcomeError::error(
                        "exception".to_string(),
                        "Failed to create access token.".to_string(),
                    )
                })?;

                Ok(Json(TokenResponse { token }).into_response())
            } else {
                Err(OperationOutcomeError::fatal(
                    "invalid".to_string(),
                    "The provided authorization code is invalid.".to_string(),
                ))
            }
        }

        _ => Err(OperationOutcomeError::fatal(
            "not-supported".to_string(),
            "The provided grant type is not supported.".to_string(),
        )),
    }
}
