use crate::{
    AppState,
    auth_n::{
        certificates::encoding_key,
        oidc::{
            extract::client_app::OIDCClientApplication,
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
use jsonwebtoken::{Algorithm, Header};
use oxidized_fhir_operation_error::{OperationOutcomeCodes, OperationOutcomeError};
use oxidized_fhir_search::SearchEngine;
use oxidized_repository::{
    Repository,
    admin::ProjectAuthAdmin,
    types::{
        ResourceId, TenantId, VersionId,
        authorization_code::{AuthorizationCode, AuthorizationCodeSearchClaims},
        user::UserRole,
    },
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
enum UserResourceTypes {
    Membership,
    ClientApplication,
    OperationDefinition,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenClaims {
    sub: ResourceId,
    exp: usize,

    tenant: TenantId,
    user_role: UserRole,
    user_resource_type: UserResourceTypes,
    user_resource_id: ResourceId,
    access_policy_version_ids: Vec<VersionId>,
}

pub async fn token<Repo: Repository + Send + Sync, Search: SearchEngine + Send + Sync>(
    _: TokenPath,
    tenant: TenantProject,
    State(state): State<Arc<AppState<Repo, Search>>>,
    current_session: Session,
    OIDCClientApplication(client_app): OIDCClientApplication,
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
            // if client_secret != client_app.secret.map(|v| v.value) {
            //     // return Err(OperationOutcomeError::error(""));
            // }

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
                        OperationOutcomeCodes::Security,
                        "Authorization code has expired.".to_string(),
                    ));
                }

                let token = jsonwebtoken::encode(
                    &Header::new(Algorithm::RS256),
                    &TokenClaims {
                        sub: ResourceId::new(code.user_id.clone()),
                        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
                        tenant: tenant.tenant,
                        user_role: UserRole::Member,
                        user_resource_type: UserResourceTypes::Membership,
                        user_resource_id: ResourceId::new(code.user_id.clone()),
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
