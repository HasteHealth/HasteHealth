use crate::{
    auth_n::{
        certificates::encoding_key,
        claims::UserTokenClaims,
        oidc::{
            code_verification,
            extract::{body::ParsedBody, client_app::find_client_app},
            schemas,
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
        AuthorId, AuthorKind, ProjectId, TenantId,
        authorization_code::{AuthorizationCodeKind, CreateAuthorizationCode},
        scope::{ClientId, CreateScope, ScopeSearchClaims, UserId},
        scopes::{OIDCScope, Scope, Scopes},
        user::UserRole,
    },
};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};

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
    refresh_token: Option<String>,
    id_token: String,
    token_type: TokenType,
    expires_in: usize,
}

struct TokenResponseArguments {
    user_id: String,
    client_id: String,
    scopes: Scopes,
    tenant: TenantId,
    project: ProjectId,
}

async fn create_token_response<Repo: Repository>(
    repo: &Repo,
    args: TokenResponseArguments,
) -> Result<TokenResponse, OperationOutcomeError> {
    let token = jsonwebtoken::encode(
        &Header::new(Algorithm::RS256),
        &UserTokenClaims {
            sub: AuthorId::new(args.user_id.clone()),
            exp: (chrono::Utc::now() + chrono::Duration::seconds(TOKEN_EXPIRATION as i64))
                .timestamp() as usize,
            aud: args.client_id.clone(),
            scope: args.scopes.clone(),
            tenant: args.tenant.clone(),
            project: Some(args.project.clone()),
            user_role: UserRole::Member,
            user_id: AuthorId::new(args.user_id.clone()),
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

    let mut response = TokenResponse {
        access_token: token.clone(),
        id_token: token,
        expires_in: TOKEN_EXPIRATION,
        refresh_token: None,
        token_type: TokenType::Bearer,
    };

    // If offline means refresh token should be generated.
    if (&args.scopes.0)
        .iter()
        .find(|s| **s == Scope::OIDC(OIDCScope::OfflineAccess))
        .is_some()
    {
        let refresh_token = ProjectAuthAdmin::create(
            repo,
            &args.tenant,
            &args.project,
            CreateAuthorizationCode {
                expires_in: Duration::from_secs(60 * 60 * 12), // 12 hours.
                kind: AuthorizationCodeKind::RefreshToken,
                user_id: args.user_id,
                client_id: Some(args.client_id),
                pkce_code_challenge: None,
                pkce_code_challenge_method: None,
                redirect_uri: None,
                meta: None,
            },
        )
        .await?;

        response.refresh_token = Some(refresh_token.code);
    }

    Ok(response)
}

async fn get_approved_scopes<Repo: Repository>(
    repo: &Repo,
    tenant: &TenantId,
    project: &ProjectId,
    user_id: UserId,
    client_id: ClientId,
) -> Result<Scopes, OperationOutcomeError> {
    let approved_scopes = ProjectAuthAdmin::<CreateScope, _, _, _, _>::search(
        repo,
        &tenant,
        &project,
        &ScopeSearchClaims {
            user_: Some(user_id),
            client: Some(client_id),
        },
    )
    .await?
    .get(0)
    .map(|s| s.scope.clone())
    .unwrap_or_else(|| Default::default());

    Ok(approved_scopes)
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
    ParsedBody(token_body): ParsedBody<schemas::token_body::OAuth2TokenBody>,
) -> Result<Response, OperationOutcomeError> {
    match token_body.grant_type {
        schemas::token_body::OAuth2TokenBodyGrantType::ClientCredentials => {
            Err(OperationOutcomeError::fatal(
                IssueType::NotSupported(None),
                "Client credentials grant type is not supported.".to_string(),
            ))
        }
        schemas::token_body::OAuth2TokenBodyGrantType::RefreshToken => {
            let client_id = token_body.client_id;
            let client_secret = token_body.client_secret;
            let refresh_token = token_body.refresh_token.ok_or_else(|| {
                OperationOutcomeError::error(
                    IssueType::Invalid(None),
                    "refresh_token is required for refresh_token grant type.".to_string(),
                )
            })?;

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
                &refresh_token,
                None,
                None,
            )
            .await?;

            if code.kind != AuthorizationCodeKind::RefreshToken {
                return Err(OperationOutcomeError::fatal(
                    IssueType::Invalid(None),
                    "Invalid refresh token.".to_string(),
                ));
            }

            let approved_scopes = get_approved_scopes(
                &*state.repo,
                &tenant,
                &project,
                UserId::new(code.user_id.clone()),
                ClientId::new(client_id.clone()),
            )
            .await?;

            ProjectAuthAdmin::<CreateAuthorizationCode, _, _, _, _>::delete(
                &*state.repo,
                &tenant,
                &project,
                &refresh_token,
            )
            .await?;

            let response = create_token_response(
                &*state.repo,
                TokenResponseArguments {
                    user_id: code.user_id.clone(),
                    client_id: client_id.clone(),
                    scopes: approved_scopes.clone(),
                    tenant: tenant.clone(),
                    project: project.clone(),
                },
            )
            .await?;

            Ok(Json(response).into_response())
        }
        schemas::token_body::OAuth2TokenBodyGrantType::AuthorizationCode => {
            let client_id = token_body.client_id;
            let client_secret = token_body.client_secret;
            let code = token_body.code.ok_or_else(|| {
                OperationOutcomeError::error(
                    IssueType::Invalid(None),
                    "code is required for authorization_code grant type.".to_string(),
                )
            })?;
            let code_verifier = token_body.code_verifier.ok_or_else(|| {
                OperationOutcomeError::error(
                    IssueType::Invalid(None),
                    "code_verifier is required for authorization_code grant type.".to_string(),
                )
            })?;
            let redirect_uri = token_body.redirect_uri.ok_or_else(|| {
                OperationOutcomeError::error(
                    IssueType::Invalid(None),
                    "redirect_uri is required for authorization_code grant type.".to_string(),
                )
            })?;

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

            if code.kind != AuthorizationCodeKind::OAuth2CodeGrant {
                return Err(OperationOutcomeError::fatal(
                    IssueType::Invalid(None),
                    "Invalid authorization code.".to_string(),
                ));
            }

            let approved_scopes = get_approved_scopes(
                &*state.repo,
                &tenant,
                &project,
                UserId::new(code.user_id.clone()),
                ClientId::new(client_id.clone()),
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

            let response = create_token_response(
                &*state.repo,
                TokenResponseArguments {
                    user_id: code.user_id.clone(),
                    client_id: client_id.clone(),
                    scopes: approved_scopes.clone(),
                    tenant: tenant.clone(),
                    project: project.clone(),
                },
            )
            .await?;

            Ok(Json(response).into_response())
        }
    }
}
