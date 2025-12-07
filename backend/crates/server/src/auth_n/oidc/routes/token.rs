use crate::{
    auth_n::{
        certificates::encoding_key,
        oidc::{
            code_verification,
            error::{OIDCError, OIDCErrorCode},
            extract::{body::ParsedBody, client_app::find_client_app},
            routes::scope::verify_requested_scope_is_subset,
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
use axum_extra::{TypedHeader, extract::Cached, headers::UserAgent, routing::TypedPath};
use haste_fhir_client::{
    request::FHIRSearchTypeRequest,
    url::{Parameter, ParsedParameter, ParsedParameters},
};
use haste_fhir_model::r4::generated::{
    resources::{ClientApplication, ResourceType},
    terminology::ClientapplicationGrantType,
};
use haste_fhir_search::{SearchEngine, SearchRequest};
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{
    AuthorId, AuthorKind, ProjectId, TenantId, UserRole, VersionId,
    claims::UserTokenClaims,
    scopes::{OIDCScope, Scope, Scopes},
};
use haste_repository::{
    Repository,
    admin::{ProjectAuthAdmin, TenantAuthAdmin},
    types::{
        SupportedFHIRVersions,
        authorization_code::{
            AuthorizationCodeKind, AuthorizationCodeSearchClaims, CreateAuthorizationCode,
        },
        scope::{ClientId, CreateScope, ScopeSearchClaims, UserId},
        user::{User, UserRole as RepoUserRole},
    },
};
use jsonwebtoken::{Algorithm, Header};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{sync::Arc, time::Duration};

#[derive(TypedPath)]
#[typed_path("/token")]
pub struct TokenPath;

#[derive(Serialize, Deserialize, Debug)]
pub enum TokenType {
    Bearer,
}

pub static TOKEN_EXPIRATION: usize = 7200; // 2 hours 

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenResponse {
    pub access_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token: Option<String>,
    token_type: TokenType,
    expires_in: usize,
}

struct TokenResponseArguments {
    user_id: String,
    user_role: UserRole,
    user_kind: AuthorKind,
    client_id: String,
    scopes: Scopes,
    tenant: TenantId,
    project: ProjectId,
    membership: Option<String>,
    access_policy_version_ids: Vec<VersionId>,
}

async fn create_token_response<Repo: Repository>(
    user_agent: &Option<TypedHeader<UserAgent>>,
    repo: &Repo,
    client_app: &ClientApplication,
    grant_type_used: &schemas::token_body::OAuth2TokenBodyGrantType,
    args: TokenResponseArguments,
) -> Result<TokenResponse, OIDCError> {
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
            user_role: args.user_role,
            user_id: AuthorId::new(args.user_id.clone()),
            membership: args.membership.clone(),
            resource_type: args.user_kind,
            access_policy_version_ids: args.access_policy_version_ids,
        },
        encoding_key(),
    )
    .map_err(|_| {
        OIDCError::new(
            OIDCErrorCode::ServerError,
            Some("Failed to create access token.".to_string()),
            None,
        )
    })?;

    let mut response = TokenResponse {
        access_token: token.clone(),
        id_token: None,
        expires_in: TOKEN_EXPIRATION,
        refresh_token: None,
        token_type: TokenType::Bearer,
    };

    if args.scopes.contains_scope(&Scope::OIDC(OIDCScope::OpenId)) {
        response.id_token = Some(token);
    }

    // If offline means refresh token should be generated.
    if (&args.scopes.0)
        .iter()
        .find(|s| **s == Scope::OIDC(OIDCScope::OfflineAccess))
        .is_some()
        && client_app
            .grantType
            .iter()
            .find(|gt| {
                let discriminator = std::mem::discriminant(gt.as_ref());
                let offline_discriminator =
                    std::mem::discriminant(&ClientapplicationGrantType::Refresh_token(None));
                discriminator == offline_discriminator
            })
            .is_some()
            // Client credentials grant does not get refresh tokens. Serves no purpose and requires knowing user kind to 
            // rebuild the token.
        && *grant_type_used != schemas::token_body::OAuth2TokenBodyGrantType::ClientCredentials
    {
        let existing_refresh_tokens_for_agent =
            ProjectAuthAdmin::<CreateAuthorizationCode, _, _, _, _>::search(
                repo,
                &args.tenant,
                &args.project,
                &AuthorizationCodeSearchClaims {
                    client_id: Some(args.client_id.clone()),
                    user_id: Some(args.user_id.clone()),
                    kind: Some(AuthorizationCodeKind::RefreshToken),
                    code: None,
                    user_agent: user_agent.as_ref().map(|ua| ua.as_str().to_string()),
                    is_expired: None,
                },
            )
            .await
            .map_err(|_| {
                OIDCError::new(
                    OIDCErrorCode::ServerError,
                    Some("Failed to retrieve existing refresh tokens.".to_string()),
                    None,
                )
            })?;

        for existing_token in existing_refresh_tokens_for_agent {
            ProjectAuthAdmin::<CreateAuthorizationCode, _, _, _, _>::delete(
                repo,
                &args.tenant,
                &args.project,
                &existing_token.code,
            )
            .await
            .map_err(|_e| {
                OIDCError::new(
                    OIDCErrorCode::ServerError,
                    Some("Failed to delete existing refresh token.".to_string()),
                    None,
                )
            })?;
        }

        let refresh_token = ProjectAuthAdmin::create(
            repo,
            &args.tenant,
            &args.project,
            CreateAuthorizationCode {
                membership: args.membership,
                user_id: args.user_id,
                expires_in: Duration::from_secs(60 * 60 * 12), // 12 hours.
                kind: AuthorizationCodeKind::RefreshToken,
                client_id: Some(args.client_id),
                pkce_code_challenge: None,
                pkce_code_challenge_method: None,
                redirect_uri: None,
                meta: Some(sqlx::types::Json(json!({
                    "user_agent": user_agent.as_ref().map(|ua| ua.to_string()),
                }))),
            },
        )
        .await
        .map_err(|_e| {
            OIDCError::new(
                OIDCErrorCode::ServerError,
                Some("Failed to create refresh token.".to_string()),
                None,
            )
        })?;

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
) -> Result<Scopes, OIDCError> {
    let approved_scopes = ProjectAuthAdmin::<CreateScope, _, _, _, _>::search(
        repo,
        &tenant,
        &project,
        &ScopeSearchClaims {
            user_: Some(user_id),
            client: Some(client_id),
        },
    )
    .await
    .map_err(|_e| {
        OIDCError::new(
            OIDCErrorCode::ServerError,
            Some("Failed to retrieve user's approved scopes.".to_string()),
            None,
        )
    })?
    .get(0)
    .map(|s| s.scope.clone())
    .unwrap_or_else(|| Default::default());

    Ok(approved_scopes)
}

fn validate_client_grant_type(
    client_app: &ClientApplication,
    grant_type: &ClientapplicationGrantType,
) -> Result<(), OIDCError> {
    if client_app
        .grantType
        .iter()
        .find(|gt| {
            let discriminator = std::mem::discriminant(gt.as_ref());
            let requested_discriminator = std::mem::discriminant(grant_type);
            discriminator == requested_discriminator
        })
        .is_none()
    {
        return Err(OIDCError::new(
            OIDCErrorCode::AccessDenied,
            Some("Client application is not authorized for the requested grant type.".to_string()),
            None,
        ));
    }

    Ok(())
}

fn verify_client(
    client_app: &ClientApplication,
    token_request_body: &schemas::token_body::OAuth2TokenBody,
) -> Result<(), OIDCError> {
    // Verify the grant types align
    match token_request_body.grant_type {
        schemas::token_body::OAuth2TokenBodyGrantType::ClientCredentials => {
            validate_client_grant_type(
                client_app,
                &ClientapplicationGrantType::Client_credentials(None),
            )?;
        }
        schemas::token_body::OAuth2TokenBodyGrantType::RefreshToken => {
            validate_client_grant_type(
                client_app,
                &ClientapplicationGrantType::Refresh_token(None),
            )?;
        }
        schemas::token_body::OAuth2TokenBodyGrantType::AuthorizationCode => {
            validate_client_grant_type(
                client_app,
                &ClientapplicationGrantType::Authorization_code(None),
            )?;
        }
    }

    if client_app.id.as_ref() != Some(&token_request_body.client_id) {
        return Err(OIDCError::new(
            OIDCErrorCode::AccessDenied,
            Some("Invalid credentials".to_string()),
            None,
        ));
    }

    if client_app
        .secret
        .as_ref()
        .and_then(|s| s.value.as_ref().map(String::as_str))
        != token_request_body
            .client_secret
            .as_ref()
            .map(String::as_str)
    {
        return Err(OIDCError::new(
            OIDCErrorCode::AccessDenied,
            Some("Invalid credentials".to_string()),
            None,
        ));
    }

    Ok(())
}

async fn find_users_access_policy_version_ids<Search: SearchEngine>(
    search: &Search,
    tenant: &TenantId,
    project: &ProjectId,
    user_id: &str,
    user_type: &ResourceType,
) -> Result<Vec<VersionId>, OIDCError> {
    let access_policies = search
        .search(
            &SupportedFHIRVersions::R4,
            &tenant,
            &project,
            SearchRequest::TypeSearch(&FHIRSearchTypeRequest {
                resource_type: ResourceType::AccessPolicyV2,
                parameters: ParsedParameters::new(vec![ParsedParameter::Resource(Parameter {
                    name: "link".to_string(),
                    value: vec![format!("{}/{}", user_type.as_ref(), user_id)],
                    modifier: None,
                    chains: None,
                })]),
            }),
            None,
        )
        .await
        .map_err(|_e| {
            OIDCError::new(
                OIDCErrorCode::ServerError,
                Some("Failed to search for user's access policies.".to_string()),
                None,
            )
        })?;

    Ok(access_policies
        .entries
        .into_iter()
        .map(|ap| ap.version_id)
        .collect())
}

#[derive(PartialEq, Eq)]
pub enum ClientCredentialsMethod {
    BasicAuth,
    Body,
}

pub async fn client_credentials_to_token_response<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    state: &AppState<Repo, Search, Terminology>,
    tenant: &TenantId,
    project: &ProjectId,
    user_agent: &Option<TypedHeader<UserAgent>>,
    token_body: &schemas::token_body::OAuth2TokenBody,
    method: ClientCredentialsMethod,
) -> Result<TokenResponse, OIDCError> {
    let client_id = &token_body.client_id;
    let client_app =
        find_client_app(state, tenant.clone(), project.clone(), client_id.clone()).await?;

    verify_client(&client_app, &token_body)?;

    // Allow basic auth if client app allows grant.
    if method == ClientCredentialsMethod::BasicAuth {
        validate_client_grant_type(
            &client_app,
            &ClientapplicationGrantType::Client_credentials(None),
        )?;
    }

    let client_app_scopes = client_app
        .scope
        .as_ref()
        .and_then(|s| s.value.as_ref().map(String::as_str))
        .unwrap_or_default();

    let requested_scopes = Scopes::from(
        token_body
            .scope
            .clone()
            .unwrap_or_else(|| client_app_scopes.to_string()),
    );

    verify_requested_scope_is_subset(
        &requested_scopes,
        &Scopes::try_from(client_app_scopes).map_err(|_| {
            OIDCError::new(
                OIDCErrorCode::InvalidScope,
                Some("Client application's configured scopes are invalid.".to_string()),
                None,
            )
        })?,
    )?;

    let response = create_token_response(
        user_agent,
        &*state.repo,
        &client_app,
        &token_body.grant_type,
        TokenResponseArguments {
            user_id: client_app.id.clone().unwrap_or_default(),
            user_role: UserRole::Member,
            user_kind: AuthorKind::ClientApplication,
            client_id: client_app.id.clone().unwrap_or_default(),
            scopes: requested_scopes,
            tenant: tenant.clone(),
            project: project.clone(),
            membership: None,
            access_policy_version_ids: find_users_access_policy_version_ids(
                state.search.as_ref(),
                &tenant,
                &project,
                client_id,
                &ResourceType::ClientApplication,
            )
            .await?,
        },
    )
    .await?;

    Ok(response)
}

pub async fn token<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: TokenPath,
    user_agent: Option<TypedHeader<UserAgent>>,
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    Cached(ProjectIdentifier { project }): Cached<ProjectIdentifier>,
    State(state): State<Arc<AppState<Repo, Search, Terminology>>>,
    ParsedBody(token_body): ParsedBody<schemas::token_body::OAuth2TokenBody>,
) -> Result<Response, OIDCError> {
    match &token_body.grant_type {
        schemas::token_body::OAuth2TokenBodyGrantType::ClientCredentials => {
            let response = client_credentials_to_token_response(
                &*state,
                &tenant,
                &project,
                &user_agent,
                &token_body,
                ClientCredentialsMethod::Body,
            )
            .await?;

            Ok(Json(response).into_response())
        }
        schemas::token_body::OAuth2TokenBodyGrantType::RefreshToken => {
            let client_id = &token_body.client_id;
            let refresh_token = &token_body.refresh_token.as_ref().ok_or_else(|| {
                OIDCError::new(
                    OIDCErrorCode::InvalidRequest,
                    Some("refresh_token is required for refresh_token grant type.".to_string()),
                    token_body.redirect_uri.clone(),
                )
            })?;

            let client_app =
                find_client_app(&state, tenant.clone(), project.clone(), client_id.clone()).await?;

            verify_client(&client_app, &token_body)?;

            let code = code_verification::retrieve_and_verify_code(
                &*state.repo,
                &tenant,
                &project,
                &client_app,
                &refresh_token,
                None,
                None,
            )
            .await
            .map_err(|_| {
                OIDCError::new(
                    OIDCErrorCode::InvalidGrant,
                    Some("Invalid refresh token.".to_string()),
                    token_body.redirect_uri.clone(),
                )
            })?;

            if code.kind != AuthorizationCodeKind::RefreshToken {
                return Err(OIDCError::new(
                    OIDCErrorCode::InvalidGrant,
                    Some("Invalid refresh token.".to_string()),
                    token_body.redirect_uri.clone(),
                ));
            }

            if code.is_expired.unwrap_or(true) {
                return Err(OIDCError::new(
                    OIDCErrorCode::InvalidGrant,
                    Some("Refresh token has expired.".to_string()),
                    token_body.redirect_uri.clone(),
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
            .await
            .map_err(|_e| {
                OIDCError::new(
                    OIDCErrorCode::ServerError,
                    Some("Failed to delete used refresh token.".to_string()),
                    token_body.redirect_uri.clone(),
                )
            })?;

            let user =
                TenantAuthAdmin::<_, User, _, _, _>::read(&*state.repo, &tenant, &code.user_id)
                    .await
                    .map_err(|_e| {
                        OIDCError::new(
                            OIDCErrorCode::ServerError,
                            Some("Failed to retrieve user.".to_string()),
                            token_body.redirect_uri.clone(),
                        )
                    })?;

            let response = create_token_response(
                &user_agent,
                &*state.repo,
                &client_app,
                &token_body.grant_type,
                TokenResponseArguments {
                    user_id: code.user_id,
                    user_kind: AuthorKind::Membership,
                    user_role: match user.map(|u| u.role) {
                        Some(RepoUserRole::Admin) => UserRole::Admin,
                        Some(RepoUserRole::Member) => UserRole::Member,
                        Some(RepoUserRole::Owner) => UserRole::Owner,
                        None => UserRole::Member,
                    },
                    client_id: client_id.clone(),
                    scopes: approved_scopes.clone(),
                    tenant: tenant.clone(),
                    project: project.clone(),
                    access_policy_version_ids: match code.membership.as_ref() {
                        Some(membership) => {
                            find_users_access_policy_version_ids(
                                state.search.as_ref(),
                                &tenant,
                                &project,
                                &membership,
                                &ResourceType::Membership,
                            )
                            .await?
                        }
                        None => vec![],
                    },
                    membership: code.membership,
                },
            )
            .await?;

            Ok(Json(response).into_response())
        }
        schemas::token_body::OAuth2TokenBodyGrantType::AuthorizationCode => {
            let client_id = &token_body.client_id;
            let code = token_body.code.as_ref().ok_or_else(|| {
                OIDCError::new(
                    OIDCErrorCode::InvalidRequest,
                    Some("code is required for authorization_code grant type.".to_string()),
                    None,
                )
            })?;
            let code_verifier = token_body.code_verifier.as_ref().ok_or_else(|| {
                OIDCError::new(
                    OIDCErrorCode::InvalidRequest,
                    Some(
                        "code_verifier is required for authorization_code grant type.".to_string(),
                    ),
                    None,
                )
            })?;
            let redirect_uri = token_body.redirect_uri.as_ref().ok_or_else(|| {
                OIDCError::new(
                    OIDCErrorCode::InvalidRequest,
                    Some("redirect_uri is required for authorization_code grant type.".to_string()),
                    None,
                )
            })?;

            let client_app =
                find_client_app(&state, tenant.clone(), project.clone(), client_id.clone()).await?;

            verify_client(&client_app, &token_body)?;

            let code = code_verification::retrieve_and_verify_code(
                &*state.repo,
                &tenant,
                &project,
                &client_app,
                &code,
                Some(&redirect_uri),
                Some(&code_verifier),
            )
            .await
            .map_err(|_| {
                OIDCError::new(
                    OIDCErrorCode::AccessDenied,
                    Some("Invalid authorization code.".to_string()),
                    None,
                )
            })?;

            if code.kind != AuthorizationCodeKind::OAuth2CodeGrant {
                return Err(OIDCError::new(
                    OIDCErrorCode::InvalidGrant,
                    Some("Invalid authorization code.".to_string()),
                    None,
                ));
            }

            if code.is_expired.unwrap_or(true) {
                return Err(OIDCError::new(
                    OIDCErrorCode::AccessDenied,
                    Some("Authorization code has expired.".to_string()),
                    None,
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
            .await
            .map_err(|_e| {
                OIDCError::new(
                    OIDCErrorCode::ServerError,
                    Some("Failed to delete used authorization code.".to_string()),
                    None,
                )
            })?;

            let user =
                TenantAuthAdmin::<_, User, _, _, _>::read(&*state.repo, &tenant, &code.user_id)
                    .await
                    .map_err(|_e| {
                        OIDCError::new(
                            OIDCErrorCode::ServerError,
                            Some("Failed to retrieve user.".to_string()),
                            None,
                        )
                    })?;

            let response = create_token_response(
                &user_agent,
                &*state.repo,
                &client_app,
                &token_body.grant_type,
                TokenResponseArguments {
                    user_id: code.user_id,
                    user_kind: AuthorKind::Membership,
                    user_role: match user.map(|u| u.role) {
                        Some(RepoUserRole::Admin) => UserRole::Admin,
                        Some(RepoUserRole::Member) => UserRole::Member,
                        Some(RepoUserRole::Owner) => UserRole::Owner,
                        None => UserRole::Member,
                    },
                    client_id: client_id.clone(),
                    scopes: approved_scopes.clone(),
                    tenant: tenant.clone(),
                    project: project.clone(),
                    access_policy_version_ids: match code.membership.as_ref() {
                        Some(membership) => {
                            find_users_access_policy_version_ids(
                                state.search.as_ref(),
                                &tenant,
                                &project,
                                &membership,
                                &ResourceType::Membership,
                            )
                            .await?
                        }
                        None => vec![],
                    },
                    membership: code.membership,
                },
            )
            .await?;

            Ok(Json(response).into_response())
        }
    }
}
