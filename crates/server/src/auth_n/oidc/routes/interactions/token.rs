use crate::{
    AppState,
    auth_n::{
        oidc::{
            middleware::OIDCParameters,
            schemas::{self, token_body::OAuth2TokenBody},
        },
        session,
    },
    extract::path_tenant::TenantProject,
};
use axum::{Extension, Json, extract::State, http::Response, response::Redirect};
use axum_extra::routing::TypedPath;
use jsonwebtoken::{Algorithm, Header};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::SearchEngine;
use oxidized_repository::{
    Repository,
    admin::ProjectAuthAdmin,
    types::authorization_code::{AuthorizationCode, AuthorizationCodeSearchClaims},
};
use std::sync::Arc;
use tower_sessions::Session;

#[derive(TypedPath)]
#[typed_path("/token")]
pub struct Token;

pub struct TokenResponse {
    token: String,
}

pub async fn token<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
>(
    _: Token,
    tenant: TenantProject,
    State(state): State<Arc<AppState<Repo, Search>>>,
    Extension(oidc_params): Extension<OIDCParameters>,
    current_session: Session,
    Json(token_body): Json<schemas::token_body::OAuth2TokenBody>,
) -> Result<Response<Json<TokenResponse>>, OperationOutcomeError> {
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

                let mut header = Header::new(Algorithm::RS256);
                // jsonwebtoken::encode(header, claims, key)
                todo!();
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
