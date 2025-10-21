use crate::{
    auth_n::{
        certificates::encoding_key,
        claims::UserTokenClaims,
        oidc::{
            code_verification,
            extract::client_app::find_client_app,
            schemas::{self, token_body::OAuth2TokenBody},
        },
    },
    extract::path_tenant::{ProjectIdentifier, TenantIdentifier},
    services::AppState,
};
use axum::{
    Json,
    extract::State,
    response::{IntoResponse, Response},
};
use axum_extra::routing::TypedPath;
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
        authorization_code::CreateAuthorizationCode,
        scope::{ClientId, CreateScope, ScopeSearchClaims, UserId},
        user::UserRole,
    },
};
use serde::{Deserialize, Serialize};
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

pub async fn token<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: TokenPath,
    TenantIdentifier { tenant }: TenantIdentifier,
    ProjectIdentifier { project }: ProjectIdentifier,
    State(state): State<Arc<AppState<Repo, Search, Terminology>>>,
    Json(token_body): Json<schemas::token_body::OAuth2TokenBody>,
) -> Result<Response, OperationOutcomeError> {
    match token_body {
        OAuth2TokenBody::ClientCredentials {
            client_id,
            client_secret,
        } => Err(OperationOutcomeError::fatal(
            IssueType::NotSupported(None),
            "Client credentials grant type is not supported.".to_string(),
        )),
        OAuth2TokenBody::RefreshToken {
            client_id,
            client_secret,
            refresh_token,
            scope,
        } => Err(OperationOutcomeError::fatal(
            IssueType::NotSupported(None),
            "Refresh grant type is not supported.".to_string(),
        )),
        OAuth2TokenBody::AuthorizationCode {
            client_id,
            client_secret,
            code,
            code_verifier,
            redirect_uri,
        } => {
            let client_app =
                find_client_app(&state, tenant.clone(), project.clone(), client_id.clone()).await?;

            if client_secret.as_ref().map(String::as_str)
                != client_app
                    .secret
                    .as_ref()
                    .and_then(|v| v.value.as_ref().map(String::as_str))
            {
                return Err(OperationOutcomeError::error(
                    IssueType::Security(None),
                    "Invalid client secret".to_string(),
                ));
            }

            let code = code_verification::retrieve_and_verify_code(
                &*state.repo,
                &tenant,
                &project,
                &client_app,
                &code,
                Some(&redirect_uri),
                Some(&code_verifier),
            )
            .await?;

            let approved_scopes = ProjectAuthAdmin::<CreateScope, _, _, _, _>::search(
                &*state.repo,
                &tenant,
                &project,
                &ScopeSearchClaims {
                    user_: Some(UserId::new(code.user_id.clone())),
                    client: Some(ClientId::new(client_id.clone())),
                },
            )
            .await?;

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
                    exp: (chrono::Utc::now() + chrono::Duration::seconds(TOKEN_EXPIRATION as i64))
                        .timestamp() as usize,
                    aud: client_id,
                    scope: approved_scopes
                        .get(0)
                        .map(|s| s.scope.clone())
                        .unwrap_or_else(|| Default::default()),
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
        }
    }
}
