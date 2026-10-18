use crate::{
    auth_n::{
        certificates::get_certification_provider,
        constant_time,
        oidc::{
            code_verification,
            error::{OIDCError, OIDCErrorCode},
            extract::{body::OAuthTokenBody, client_app::find_client_app},
            routes::scope::verify_requested_scope_is_subset,
            schemas,
        },
    },
    config::ServerConfig,
    extract::path_tenant::{ProjectIdentifier, TenantIdentifier},
    route_path::{api_fhir_root_url, issuer_url},
    services::ServerState,
};
use axum::{
    Json,
    extract::State,
    response::{IntoResponse, Response},
};
use axum_extra::{TypedHeader, extract::Cached, headers::UserAgent, routing::TypedPath};
use haste_fhir_client::{
    FHIRClient,
    request::{FHIRReadRequest, FHIRRequest, FHIRResponse, FHIRSearchTypeRequest, SearchRequest},
};
use haste_fhir_model::r4::generated::{
    resources::{ClientApplication, Resource, ResourceType},
    terminology::{BoundCode, ClientapplicationGrantType},
    types::FHIRUrl,
};
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{
    AuthorId, AuthorKind, ProjectId, TenantId, UserRole, VersionId,
    claims::{IdTokenClaims, SubscriptionTier, UserTokenClaims},
    scopes::{
        LaunchSystemScope, LaunchType, LaunchTypeScope, OIDCScope, SMARTResourceScope, Scope,
        Scopes, SmartResourceScopePermission, SmartResourceScopePermissions,
        SmartResourceScopeUser, SmartScope,
    },
};
use haste_repository::{
    Repository,
    admin::{ProjectModelAdmin, TenantModelAdmin},
    types::{
        SupportedFHIRVersions,
        authorization_code::{
            AuthorizationCode, AuthorizationCodeKind, AuthorizationCodeSearchClaims,
            CreateAuthorizationCode,
        },
        project::CreateProject,
        scope::{ClientId, CreateScope, ScopeSearchClaims, UserId},
        tenant::CreateTenant,
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
pub static REFRESH_TOKEN_EXPIRATION: usize = 43200; // 12 hours

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenResponse {
    pub access_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token: Option<String>,
    token_type: TokenType,
    expires_in: usize,
    /// The scopes granted, which SMART lets differ from those requested.
    #[serde(default)]
    scope: String,
    /// SMART launch context: the Patient the `patient/` scopes are bound to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    patient: Option<String>,
    /// SMART launch context: the Encounter an EHR launch was started in.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    encounter: Option<String>,
    /// SMART launch context: whether the app should show which patient it is
    /// showing. Haste Health opens a launched app on its own, with no patient
    /// banner around it, so this is true for every EHR launch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    need_patient_banner: Option<bool>,
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
    fhir_user: Option<FHIRUrl>,
    patient: Option<String>,
    encounter: Option<String>,
    /// The SMART EHR launch the token comes from, kept on its refresh token.
    launch: Option<LaunchContext>,
    /// The user's address and whether they verified it, for the id token's
    /// `email` claims. A client application has neither.
    email: Option<String>,
    email_verified: bool,
}

/// What a token says about the user it is issued to.
struct TokenUser {
    role: UserRole,
    email: Option<String>,
    email_verified: bool,
}

impl TokenUser {
    /// A user that could not be read is issued the least privileged role and
    /// no address.
    fn of(user: Option<User>) -> TokenUser {
        match user {
            Some(user) => TokenUser {
                role: match user.role {
                    RepoUserRole::Admin => UserRole::Admin,
                    RepoUserRole::Member => UserRole::Member,
                    RepoUserRole::Owner => UserRole::Owner,
                },
                email: user.email,
                email_verified: user.email_verified,
            },
            None => TokenUser {
                role: UserRole::Member,
                email: None,
                email_verified: false,
            },
        }
    }
}

/// The context `$launch` saved and authorize copied onto the authorization
/// code, under `meta.launch`. A refresh token keeps it there too, so a
/// refreshed token stays bound to the launch.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct LaunchContext {
    patient: String,
    encounter: Option<String>,
}

impl LaunchContext {
    /// The launch context an authorization code or refresh token carries, if
    /// it came from an EHR launch.
    fn of(code: &AuthorizationCode) -> Option<LaunchContext> {
        let launch = code.meta.as_ref()?.0.get("launch")?.clone();
        serde_json::from_value(launch).ok()
    }
}

/// Whether `scope` needs a patient in context.
fn is_patient_bound_scope(scope: &Scope) -> bool {
    matches!(
        scope,
        Scope::SMART(SmartScope::Resource(SMARTResourceScope {
            user: SmartResourceScopeUser::Patient,
            ..
        })) | Scope::SMART(SmartScope::LaunchType(LaunchTypeScope {
            launch_type: LaunchType::Patient,
        }))
    )
}

/// What the membership link, and an EHR launch, bind a token to.
struct SmartContext {
    fhir_user: Option<FHIRUrl>,
    patient: Option<String>,
    encounter: Option<String>,
    scopes: Scopes,
}

/// Filters SMART resource scopes to only include read and search permissions.
fn smart_read_only_scopes(scope: SMARTResourceScope) -> Option<SMARTResourceScope> {
    let kept: Vec<_> = [
        SmartResourceScopePermission::Read,
        SmartResourceScopePermission::Search,
    ]
    .into_iter()
    .filter(|permission| scope.permissions.has_permission(permission))
    .collect();

    (!kept.is_empty()).then(|| SMARTResourceScope {
        permissions: SmartResourceScopePermissions::new(kept),
        ..scope
    })
}

/// How a requested `scope` is issued. A patient-bound scope needs a patient
/// in context and is then read-only; without one it is dropped rather than
/// issued unbound. `launch` needs an EHR launch and `launch/encounter` an
/// encounter in context. Anything else is issued as requested.
fn issued_scope(
    scope: Scope,
    has_patient: bool,
    ehr_launch: bool,
    has_encounter: bool,
) -> Option<Scope> {
    match scope {
        Scope::SMART(SmartScope::LaunchSystem(_)) => ehr_launch.then_some(scope),
        Scope::SMART(SmartScope::LaunchType(LaunchTypeScope {
            launch_type: LaunchType::Encounter,
        })) => has_encounter.then_some(scope),
        // For patient-level resource scopes, only issue them if a patient is in context, and make them read-only.
        Scope::SMART(SmartScope::Resource(resource))
            if resource.user == SmartResourceScopeUser::Patient =>
        {
            has_patient
                .then(|| smart_read_only_scopes(resource))
                .flatten()
                .map(|resource| Scope::SMART(SmartScope::Resource(resource)))
        }
        Scope::SMART(SmartScope::LaunchType(LaunchTypeScope {
            launch_type: LaunchType::Patient,
        })) => has_patient.then_some(scope),
        scope => Some(scope),
    }
}

/// Binds `scopes` to the resource the membership links to: `fhirUser` names
/// it, and a linked Patient is the patient in context. An EHR launch's
/// patient comes first: it is the patient the user launched the app for. The
/// response's `scope` tells the client what was issued.
///
/// # Errors
///
/// A user who is a patient may only launch an app for themselves, so a
/// launch for another patient is refused.
fn bind_smart_context(
    scopes: Scopes,
    link: Option<String>,
    launch: Option<&LaunchContext>,
) -> Result<SmartContext, OIDCError> {
    let linked_patient = link
        .as_deref()
        .and_then(|link| link.strip_prefix("Patient/"))
        .filter(|id| !id.is_empty())
        .map(str::to_string);

    let patient = match (launch, linked_patient) {
        (Some(launch), Some(linked)) if linked != launch.patient => {
            return Err(OIDCError::new(
                OIDCErrorCode::InvalidGrant,
                Some(
                    "The app was launched for a patient other than the signed-in patient."
                        .to_string(),
                ),
                None,
            ));
        }
        (Some(launch), _) => Some(launch.patient.clone()),
        (None, linked) => linked,
    };
    let encounter = launch.and_then(|launch| launch.encounter.clone());

    let fhir_user = if scopes.contains_scope(&Scope::SMART(SmartScope::FHIRUser)) {
        link.map(|reference| FHIRUrl {
            value: Some(reference),
            ..Default::default()
        })
    } else {
        None
    };

    let scopes = Scopes(
        scopes
            .0
            .into_iter()
            .filter_map(|scope| {
                issued_scope(
                    scope,
                    patient.is_some(),
                    launch.is_some(),
                    encounter.is_some(),
                )
            })
            .collect(),
    );

    Ok(SmartContext {
        fhir_user,
        patient,
        encounter,
        scopes,
    })
}

async fn create_token_response<Repo: Repository>(
    config: &ServerConfig,
    user_agent: &Option<TypedHeader<UserAgent>>,
    repo: &Repo,
    client_app: &ClientApplication,
    grant_type_used: &schemas::token_body::OAuth2TokenBodyGrantType,
    args: TokenResponseArguments,
) -> Result<TokenResponse, OIDCError> {
    // Per SMART on FHIR, `aud` identifies the FHIR resource server the token is
    // valid for, binding the token to this tenant/project's FHIR endpoint.
    let audience = api_fhir_root_url(&config.api_uri, &args.tenant, &args.project)
        .map_err(|e| OIDCError::new(OIDCErrorCode::ServerError, Some(e.to_string()), None))?;

    let cert_provider = get_certification_provider(config);
    let encoding_key = cert_provider.encoding_key().map_err(|_e| {
        OIDCError::new(
            OIDCErrorCode::ServerError,
            Some("Failed to create access token. No encoding key available.".to_string()),
            None,
        )
    })?;

    let tenant = TenantModelAdmin::<CreateTenant, _, _, _, _>::read(
        repo,
        &TenantId::System,
        &args.tenant.as_ref().to_string(),
    )
    .await
    .map_err(|_e| {
        OIDCError::new(
            OIDCErrorCode::ServerError,
            Some("Failed to retrieve tenant information.".to_string()),
            None,
        )
    })?
    .ok_or_else(|| {
        OIDCError::new(
            OIDCErrorCode::ServerError,
            Some("Tenant not found.".to_string()),
            None,
        )
    })?;

    // Carried in the token so that a request never has to look the project up
    // to know which FHIR version it is working in. A project's version cannot
    // change, so the claim cannot go stale within a token's lifetime.
    let project_fhir_version = TenantModelAdmin::<CreateProject, _, _, _, _>::read(
        repo,
        &args.tenant,
        &args.project.as_ref().to_string(),
    )
    .await
    .map_err(|_e| {
        OIDCError::new(
            OIDCErrorCode::ServerError,
            Some("Failed to retrieve project.".to_string()),
            None,
        )
    })?
    .ok_or_else(|| {
        OIDCError::new(
            OIDCErrorCode::ServerError,
            Some("Project not found.".to_string()),
            None,
        )
    })?
    .fhir_version;

    let issued_at = chrono::Utc::now().timestamp() as usize;

    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(encoding_key.kid.clone());

    let access_token = jsonwebtoken::encode(
        &header,
        &UserTokenClaims {
            sub: AuthorId::new(args.user_id.clone()),
            exp: issued_at + TOKEN_EXPIRATION,
            aud: audience.to_string(),
            scope: args.scopes.clone(),
            tenant: args.tenant.clone(),
            subscription_tier: SubscriptionTier::try_from(tenant.subscription_tier).map_err(
                |e| OIDCError::new(OIDCErrorCode::ServerError, Some(e.to_string()), None),
            )?,
            project: Some(args.project.clone()),
            user_role: args.user_role.clone(),
            user_id: AuthorId::new(args.user_id.clone()),
            membership: args.membership.clone(),
            resource_type: args.user_kind,
            access_policy_version_ids: args.access_policy_version_ids,
            fhir_user: args.fhir_user.clone(),
            patient: args.patient.clone(),
            fhir_version: project_fhir_version,
        },
        &encoding_key.encoding_key,
    )
    .map_err(|_| {
        OIDCError::new(
            OIDCErrorCode::ServerError,
            Some("Failed to create access token.".to_string()),
            None,
        )
    })?;

    let launch_granted = args.launch.is_some()
        && args
            .scopes
            .contains_scope(&Scope::SMART(SmartScope::LaunchSystem(LaunchSystemScope)));

    // An id token describes the signed-in user to the client that asked for
    // the login (OpenID Connect Core §2), so unlike the access token it is
    // addressed to that client and not to the FHIR server. A client credentials
    // grant has no user to describe.
    let id_token = if args.scopes.contains_scope(&Scope::OIDC(OIDCScope::OpenId))
        && *grant_type_used != schemas::token_body::OAuth2TokenBodyGrantType::ClientCredentials
    {
        let issuer = issuer_url(&config.api_uri, &args.tenant, &args.project)
            .map_err(|e| OIDCError::new(OIDCErrorCode::ServerError, Some(e.to_string()), None))?;
        // The email claims go with the `email` scope (OpenID Connect Core §5.4).
        let email = args
            .email
            .clone()
            .filter(|_| args.scopes.contains_scope(&Scope::OIDC(OIDCScope::Email)));

        let id_token = jsonwebtoken::encode(
            &header,
            &IdTokenClaims {
                iss: issuer.to_string(),
                sub: args.user_id.clone(),
                aud: args.client_id.clone(),
                exp: issued_at + TOKEN_EXPIRATION,
                iat: issued_at,
                email_verified: email.as_ref().map(|_| args.email_verified),
                email,
                fhir_user: args.fhir_user.clone(),
                tenant: args.tenant.clone(),
                project: args.project.clone(),
                user_role: args.user_role.clone(),
            },
            &encoding_key.encoding_key,
        )
        .map_err(|_| {
            OIDCError::new(
                OIDCErrorCode::ServerError,
                Some("Failed to create id token.".to_string()),
                None,
            )
        })?;

        Some(id_token)
    } else {
        None
    };

    let mut response = TokenResponse {
        access_token,
        id_token,
        expires_in: TOKEN_EXPIRATION,
        refresh_token: None,
        token_type: TokenType::Bearer,
        scope: String::from(args.scopes.clone()),
        // An EHR launch returns its context with the `launch` scope; otherwise
        // the patient is only returned for the scopes it is bound to.
        patient: args
            .patient
            .clone()
            .filter(|_| launch_granted || args.scopes.0.iter().any(is_patient_bound_scope)),
        encounter: args.encounter.clone().filter(|_| launch_granted),
        need_patient_banner: launch_granted.then_some(true),
    };

    // If offline means refresh token should be generated.
    if args.scopes.0
        .iter()
        .find(|s| **s == Scope::OIDC(OIDCScope::OfflineAccess))
        .is_some()
        && client_app
            .grantType
            .iter()
            .find(|gt| {
                gt == &&ClientapplicationGrantType::refresh_token()
            })
            .is_some()
            // Client credentials grant does not get refresh tokens. Serves no purpose and requires knowing user kind to 
            // rebuild the token.
        && *grant_type_used != schemas::token_body::OAuth2TokenBodyGrantType::ClientCredentials
    {
        let existing_refresh_tokens_for_agent =
            ProjectModelAdmin::<CreateAuthorizationCode, _, _, _, _>::search(
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
            ProjectModelAdmin::<CreateAuthorizationCode, _, _, _, _>::delete(
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

        let refresh_token = ProjectModelAdmin::create(
            repo,
            &args.tenant,
            &args.project,
            CreateAuthorizationCode {
                membership: args.membership,
                user_id: args.user_id,
                expires_in: Duration::from_secs(REFRESH_TOKEN_EXPIRATION as u64),
                kind: AuthorizationCodeKind::RefreshToken,
                client_id: Some(args.client_id),
                pkce_code_challenge: None,
                pkce_code_challenge_method: None,
                redirect_uri: None,
                meta: Some(sqlx::types::Json(match args.launch {
                    // The launch goes with the refresh token, so the token it
                    // is exchanged for keeps the same patient and encounter.
                    Some(launch) => json!({
                        "user_agent": user_agent.as_ref().map(|ua| ua.to_string()),
                        "launch": launch,
                    }),
                    None => json!({
                        "user_agent": user_agent.as_ref().map(|ua| ua.to_string()),
                    }),
                })),
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
    let approved_scopes = ProjectModelAdmin::<CreateScope, _, _, _, _>::search(
        repo,
        tenant,
        project,
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
    .first()
    .map(|s| s.scope.clone())
    .unwrap_or_else(Default::default);

    Ok(approved_scopes)
}

fn validate_client_grant_type(
    client_app: &ClientApplication,
    grant_type: &BoundCode<ClientapplicationGrantType>,
) -> Result<(), OIDCError> {
    if client_app
        .grantType
        .iter()
        .find(|gt| gt == &grant_type)
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
                &ClientapplicationGrantType::client_credentials(),
            )?;
            // Enforce requirement of a client_secret.
            require_client_secret(client_app)?;
        }
        schemas::token_body::OAuth2TokenBodyGrantType::RefreshToken => {
            validate_client_grant_type(client_app, &ClientapplicationGrantType::refresh_token())?;
        }
        schemas::token_body::OAuth2TokenBodyGrantType::AuthorizationCode => {
            validate_client_grant_type(
                client_app,
                &ClientapplicationGrantType::authorization_code(),
            )?;
        }
    }

    if client_app.id.as_ref() != token_request_body.client_id.as_ref() {
        return Err(OIDCError::new(
            OIDCErrorCode::AccessDenied,
            Some("Invalid credentials".to_string()),
            None,
        ));
    }

    if !constant_time::optional_str_eq(
        client_app.secret.as_ref().and_then(|s| s.value.as_deref()),
        token_request_body.client_secret.as_deref(),
    ) {
        return Err(OIDCError::new(
            OIDCErrorCode::AccessDenied,
            Some("Invalid credentials".to_string()),
            None,
        ));
    }

    Ok(())
}

/// Refuses a confidential grant for a client that has no secret stored.
fn require_client_secret(client_app: &ClientApplication) -> Result<(), OIDCError> {
    let has_secret = client_app
        .secret
        .as_ref()
        .and_then(|s| s.value.as_deref())
        .is_some_and(|secret| !secret.is_empty());

    if has_secret {
        Ok(())
    } else {
        Err(OIDCError::new(
            OIDCErrorCode::AccessDenied,
            Some("Invalid credentials".to_string()),
            None,
        ))
    }
}

async fn refresh_token_client_id<Repo: Repository>(
    repo: &Repo,
    tenant: &TenantId,
    project: &ProjectId,
    refresh_token: &str,
) -> Result<String, OIDCError> {
    let invalid = || {
        OIDCError::new(
            OIDCErrorCode::InvalidGrant,
            Some("Invalid refresh token.".to_string()),
            None,
        )
    };

    ProjectModelAdmin::<CreateAuthorizationCode, _, _, _, _>::search(
        repo,
        tenant,
        project,
        &AuthorizationCodeSearchClaims {
            client_id: None,
            code: Some(refresh_token.to_string()),
            kind: Some(AuthorizationCodeKind::RefreshToken),
            user_id: None,
            user_agent: None,
            is_expired: None,
        },
    )
    .await
    .map_err(|_| invalid())?
    .pop()
    .and_then(|code| code.client_id)
    .ok_or_else(invalid)
}

async fn search_version_ids<Search: SearchEngine>(
    search: &Search,
    tenant: &TenantId,
    project: &ProjectId,
    resource_type: ResourceType,
    parameter: &str,
    values: Vec<String>,
) -> Result<Vec<VersionId>, OIDCError> {
    let result = search
        .search(
            &SupportedFHIRVersions::R4,
            tenant,
            project,
            &SearchRequest::Type(FHIRSearchTypeRequest {
                resource_type,
                parameters: vec![(parameter.to_string(), values)].into(),
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

    Ok(result.entries.into_iter().map(|e| e.version_id).collect())
}

/// Version ids of the access policies that apply to `user_type/user_id`.
///
/// Policies are assigned through `AccessPolicyV2Assignment` resources. Policies
/// that still name the user in the deprecated `AccessPolicyV2.target` are
/// included too, so existing policies keep applying until they are migrated.
async fn find_users_access_policy_version_ids<Repo: Repository, Search: SearchEngine>(
    repo: &Repo,
    search: &Search,
    tenant: &TenantId,
    project: &ProjectId,
    user_id: &str,
    user_type: &ResourceType,
) -> Result<Vec<VersionId>, OIDCError> {
    let user_reference = format!("{}/{}", user_type.as_ref(), user_id);

    let assignment_version_ids = search_version_ids(
        search,
        tenant,
        project,
        ResourceType::AccessPolicyV2Assignment,
        "link",
        vec![user_reference.clone()],
    )
    .await?;

    let assigned_policy_ids = repo
        .read_by_version_ids(
            tenant,
            project,
            &assignment_version_ids.iter().collect::<Vec<_>>(),
            haste_repository::fhir::CachePolicy::NoCache,
        )
        .await
        .map_err(|_e| {
            OIDCError::new(
                OIDCErrorCode::ServerError,
                Some("Failed to read user's access policy assignments.".to_string()),
                None,
            )
        })?
        .into_iter()
        .filter_map(|resource| match resource {
            Resource::AccessPolicyV2Assignment(assignment) => assignment
                .accessPolicy
                .reference
                .and_then(|reference| reference.value)
                .and_then(|reference| {
                    reference
                        .strip_prefix("AccessPolicyV2/")
                        .map(ToString::to_string)
                }),
            _ => None,
        })
        .collect::<Vec<_>>();

    let mut version_ids = if assigned_policy_ids.is_empty() {
        vec![]
    } else {
        search_version_ids(
            search,
            tenant,
            project,
            ResourceType::AccessPolicyV2,
            "_id",
            assigned_policy_ids,
        )
        .await?
    };

    // Deprecated: AccessPolicyV2.target.link.
    version_ids.extend(
        search_version_ids(
            search,
            tenant,
            project,
            ResourceType::AccessPolicyV2,
            "link",
            vec![user_reference],
        )
        .await?,
    );

    let mut seen = std::collections::HashSet::new();
    version_ids.retain(|version_id| seen.insert(version_id.clone()));

    Ok(version_ids)
}

/// The resource the membership links to (`Patient/123`), if any.
async fn membership_link<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    state: &ServerState<Repo, Search, Terminology>,
    tenant: &TenantId,
    project: &ProjectId,
    membership_id: Option<&str>,
) -> Result<Option<String>, OIDCError> {
    let Some(membership_id) = membership_id else {
        return Ok(None);
    };

    let response = state
        .fhir_client
        .request(
            Arc::new(crate::fhir_client::ServerCTX::system(
                tenant.clone(),
                project.clone(),
                state.fhir_client.clone(),
                state.rate_limit.clone(),
            )),
            FHIRRequest::Read(FHIRReadRequest {
                resource_type: ResourceType::Membership,
                id: membership_id.to_string(),
            }),
        )
        .await
        .map_err(|_| {
            OIDCError::new(
                OIDCErrorCode::ServerError,
                Some("Failed to retrieve membership resource for its link.".to_string()),
                None,
            )
        })?;

    let membership_link_reference = match response {
        FHIRResponse::Read(read_response) => {
            read_response.resource.and_then(|resource| match resource {
                Resource::Membership(membership) => membership
                    .link
                    .as_ref()
                    .and_then(|reference| reference.reference.as_ref())
                    .and_then(|reference| reference.value.clone()),
                _ => None,
            })
        }
        _ => None,
    };

    Ok(membership_link_reference)
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
    state: &ServerState<Repo, Search, Terminology>,
    tenant: &TenantId,
    project: &ProjectId,
    user_agent: &Option<TypedHeader<UserAgent>>,
    token_body: &schemas::token_body::OAuth2TokenBody,
    method: ClientCredentialsMethod,
) -> Result<TokenResponse, OIDCError> {
    let Some(client_id) = &token_body.client_id else {
        return Err(OIDCError::new(
            OIDCErrorCode::InvalidRequest,
            Some("client_id is required for client_credentials grant type.".to_string()),
            None,
        ));
    };

    let client_app =
        find_client_app(state, tenant.clone(), project.clone(), client_id.clone()).await?;

    verify_client(&client_app, token_body)?;

    // Allow basic auth if client app allows grant.
    if method == ClientCredentialsMethod::BasicAuth {
        validate_client_grant_type(&client_app, &ClientapplicationGrantType::basic_auth())?;
    }

    let client_app_scopes = client_app
        .scope
        .as_ref()
        .and_then(|s| s.value.as_deref())
        .unwrap_or_default();

    let requested = token_body
        .scope
        .clone()
        .unwrap_or_else(|| client_app_scopes.to_string());

    let requested_scopes = Scopes::try_from(requested.as_str()).map_err(|_| {
        OIDCError::new(
            OIDCErrorCode::InvalidScope,
            Some("Invalid scope requested.".to_string()),
            None,
        )
    })?;

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
        state.config.as_ref(),
        user_agent,
        &*state.repo,
        &client_app,
        &token_body.grant_type,
        TokenResponseArguments {
            user_id: client_app.id.clone().unwrap_or_default(),
            user_role: UserRole::Member,
            email: None,
            email_verified: false,
            user_kind: AuthorKind::ClientApplication,
            client_id: client_app.id.clone().unwrap_or_default(),
            // A client has no patient in context.
            scopes: bind_smart_context(requested_scopes, None, None)?.scopes,
            tenant: tenant.clone(),
            project: project.clone(),
            membership: None,
            fhir_user: None,
            patient: None,
            encounter: None,
            launch: None,
            access_policy_version_ids: find_users_access_policy_version_ids(
                state.repo.as_ref(),
                state.search.as_ref(),
                tenant,
                project,
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
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
    OAuthTokenBody(token_body): OAuthTokenBody,
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
            let refresh_token = &token_body.refresh_token.as_ref().ok_or_else(|| {
                OIDCError::new(
                    OIDCErrorCode::InvalidRequest,
                    Some("refresh_token is required for refresh_token grant type.".to_string()),
                    None,
                )
            })?;

            // A public client need not send client_id when refreshing (RFC 6749
            // section 6, and fhirclient does not by default): the refresh
            // token names the client it was issued to. A confidential client
            // still has to authenticate, which verify_client checks.
            let client_id = match &token_body.client_id {
                Some(client_id) => client_id.clone(),
                None => {
                    refresh_token_client_id(&*state.repo, &tenant, &project, refresh_token).await?
                }
            };
            let token_body = schemas::token_body::OAuth2TokenBody {
                client_id: Some(client_id.clone()),
                ..token_body.clone()
            };
            let client_id = &client_id;

            let client_app =
                find_client_app(&state, tenant.clone(), project.clone(), client_id.clone()).await?;

            verify_client(&client_app, &token_body)?;

            let code = code_verification::retrieve_and_verify_code(
                &*state.repo,
                &tenant,
                &project,
                &client_app,
                AuthorizationCodeKind::RefreshToken,
                refresh_token,
                None,
                None,
            )
            .await
            .map_err(|_e| {
                OIDCError::new(
                    OIDCErrorCode::InvalidGrant,
                    Some("Invalid refresh token.".to_string()),
                    None,
                )
            })?;

            if code.kind != AuthorizationCodeKind::RefreshToken {
                return Err(OIDCError::new(
                    OIDCErrorCode::InvalidGrant,
                    Some("Invalid refresh token.".to_string()),
                    None,
                ));
            }

            if code.is_expired.unwrap_or(true) {
                return Err(OIDCError::new(
                    OIDCErrorCode::InvalidGrant,
                    Some("Refresh token has expired.".to_string()),
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

            ProjectModelAdmin::<CreateAuthorizationCode, _, _, _, _>::delete(
                &*state.repo,
                &tenant,
                &project,
                refresh_token,
            )
            .await
            .map_err(|_e| {
                OIDCError::new(
                    OIDCErrorCode::ServerError,
                    Some("Failed to delete used refresh token.".to_string()),
                    None,
                )
            })?;

            let user =
                TenantModelAdmin::<_, User, _, _, _>::read(&*state.repo, &tenant, &code.user_id)
                    .await
                    .map_err(|_e| {
                        OIDCError::new(
                            OIDCErrorCode::ServerError,
                            Some("Failed to retrieve user.".to_string()),
                            None,
                        )
                    })?;
            let token_user = TokenUser::of(user);

            let link =
                membership_link(&state, &tenant, &project, code.membership.as_deref()).await?;
            // The launch the refresh token was issued for, so the new token
            // keeps its patient and encounter.
            let launch = LaunchContext::of(&code);
            let SmartContext {
                fhir_user,
                patient,
                encounter,
                scopes: approved_scopes,
            } = bind_smart_context(approved_scopes, link, launch.as_ref())?;

            let response = create_token_response(
                state.config.as_ref(),
                &user_agent,
                &*state.repo,
                &client_app,
                &token_body.grant_type,
                TokenResponseArguments {
                    user_id: code.user_id,
                    user_kind: AuthorKind::Membership,
                    user_role: token_user.role,
                    email: token_user.email,
                    email_verified: token_user.email_verified,
                    client_id: client_id.clone(),
                    scopes: approved_scopes.clone(),
                    tenant: tenant.clone(),
                    project: project.clone(),
                    access_policy_version_ids: match code.membership.as_ref() {
                        Some(membership) => {
                            find_users_access_policy_version_ids(
                                state.repo.as_ref(),
                                state.search.as_ref(),
                                &tenant,
                                &project,
                                membership,
                                &ResourceType::Membership,
                            )
                            .await?
                        }
                        None => vec![],
                    },
                    membership: code.membership.clone(),
                    fhir_user,
                    patient,
                    encounter,
                    launch,
                },
            )
            .await?;

            Ok(Json(response).into_response())
        }
        schemas::token_body::OAuth2TokenBodyGrantType::AuthorizationCode => {
            let Some(client_id) = &token_body.client_id else {
                return Err(OIDCError::new(
                    OIDCErrorCode::InvalidRequest,
                    Some("client_id is required for authorization_code grant type.".to_string()),
                    None,
                ));
            };

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
                AuthorizationCodeKind::OAuth2CodeGrant,
                code,
                Some(redirect_uri),
                Some(code_verifier),
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
            ProjectModelAdmin::<CreateAuthorizationCode, _, _, _, _>::delete(
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
                TenantModelAdmin::<_, User, _, _, _>::read(&*state.repo, &tenant, &code.user_id)
                    .await
                    .map_err(|_e| {
                        OIDCError::new(
                            OIDCErrorCode::ServerError,
                            Some("Failed to retrieve user.".to_string()),
                            None,
                        )
                    })?;
            let token_user = TokenUser::of(user);

            let link =
                membership_link(&state, &tenant, &project, code.membership.as_deref()).await?;
            // An EHR launch's patient and encounter, saved on the code by authorize.
            let launch = LaunchContext::of(&code);
            let SmartContext {
                fhir_user,
                patient,
                encounter,
                scopes: approved_scopes,
            } = bind_smart_context(approved_scopes, link, launch.as_ref())?;

            let response = create_token_response(
                state.config.as_ref(),
                &user_agent,
                &*state.repo,
                &client_app,
                &token_body.grant_type,
                TokenResponseArguments {
                    user_id: code.user_id,
                    user_kind: AuthorKind::Membership,
                    user_role: token_user.role,
                    email: token_user.email,
                    email_verified: token_user.email_verified,
                    client_id: client_id.clone(),
                    scopes: approved_scopes.clone(),
                    tenant: tenant.clone(),
                    project: project.clone(),
                    access_policy_version_ids: match code.membership.as_ref() {
                        Some(membership) => {
                            find_users_access_policy_version_ids(
                                state.repo.as_ref(),
                                state.search.as_ref(),
                                &tenant,
                                &project,
                                membership,
                                &ResourceType::Membership,
                            )
                            .await?
                        }
                        None => vec![],
                    },
                    membership: code.membership.clone(),
                    fhir_user,
                    patient,
                    encounter,
                    launch,
                },
            )
            .await?;

            Ok(Json(response).into_response())
        }
    }
}
