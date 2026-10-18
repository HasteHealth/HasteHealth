use crate::{
    auth_n::{
        oidc::{
            error::{OIDCError, OIDCErrorCode},
            extract::{client_app::OIDCClientApplication, scopes::Scopes},
            middleware::OIDCParameters,
            routes::scope::{ScopeForm, verify_requested_scope_is_subset},
            utilities::is_valid_redirect_url,
        },
        session,
    },
    extract::{
        csrf_token::CSRFToken,
        path_tenant::{Project, ProjectIdentifier},
    },
    route_path::api_fhir_root_url,
    services::ServerState,
    ui::{components::TenantContext, pages},
};
use axum::{
    Extension,
    extract::{OriginalUri, State},
    http::{Uri, uri::Scheme},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::{extract::Cached, routing::TypedPath};
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{
    ProjectId, SupportedFHIRVersions, TenantId,
    scopes::{LaunchSystemScope, Scope, SmartScope},
};
use haste_repository::{
    Repository,
    admin::ProjectModelAdmin,
    types::{
        authorization_code::{
            AuthorizationCodeKind, AuthorizationCodeSearchClaims, CreateAuthorizationCode,
            PKCECodeChallengeMethod,
        },
        membership::{Membership, MembershipSearchClaims},
        scope::{ClientId, CreateScope, ScopeKey, UserId},
        user::{User, UserRole},
    },
};
use serde_json::json;
use sqlx::types::Json;
use std::{sync::Arc, time::Duration};
use tower_sessions::Session;
use tracing::warn;
use url::Url;

/// Whether `aud` names this project's FHIR endpoint. The version segment is
/// optional in FHIR URLs, so `…/fhir` and `…/fhir/r4` are the same endpoint.
fn is_project_fhir_base(aud: &str, fhir_root: &Url) -> bool {
    let Ok(aud) = Url::parse(aud) else {
        return false;
    };

    if aud.query().is_some() || aud.fragment().is_some() {
        return false;
    }

    let aud = aud.as_str().trim_end_matches('/');
    let root = fhir_root.as_str().trim_end_matches('/');

    aud == root
        || SupportedFHIRVersions::ALL
            .iter()
            .any(|version| aud == format!("{root}/{version}"))
}

/// The context of the SMART EHR launch `launch` names: the patient (and
/// encounter) `$launch` saved with it.
///
/// The launch must be one the signed-in user started for this client, in this
/// project, and must not have expired. It is not used up here: an app may
/// retry, and consent sends the browser back through authorize. Anything else
/// is an `invalid_request`, reported at the client's redirect URI.
async fn resolve_launch<Repo: Repository>(
    repo: &Repo,
    tenant: &TenantId,
    project: &ProjectId,
    client_id: &str,
    user_id: &str,
    launch: &str,
    redirect_uri: &str,
) -> Result<serde_json::Value, OIDCError> {
    let invalid = |description: &str| {
        OIDCError::new(
            OIDCErrorCode::InvalidRequest,
            Some(description.to_string()),
            Some(redirect_uri.to_string()),
        )
    };

    let launches = ProjectModelAdmin::<CreateAuthorizationCode, _, _, _, _>::search(
        repo,
        tenant,
        project,
        &AuthorizationCodeSearchClaims {
            client_id: Some(client_id.to_string()),
            code: Some(launch.to_string()),
            kind: Some(AuthorizationCodeKind::SmartLaunch),
            user_id: Some(user_id.to_string()),
            user_agent: None,
            is_expired: None,
        },
    )
    .await
    .map_err(|_| {
        OIDCError::new(
            OIDCErrorCode::ServerError,
            Some("Failed to look up the launch.".to_string()),
            Some(redirect_uri.to_string()),
        )
    })?;

    // One message for every way a launch can fail to match, so the response
    // does not tell a caller which launch values exist.
    let Some(found) = launches.into_iter().next() else {
        return Err(invalid(
            "Unknown launch: it was not started by this user for this app.",
        ));
    };

    if found.is_expired.unwrap_or(true) {
        return Err(invalid("The launch has expired. Start the app again."));
    }

    Ok(found.meta.map(|meta| meta.0).unwrap_or_default())
}

pub fn redirect_authorize_uri(uri: &OriginalUri, replace_path: &str) -> String {
    uri.path()
        .to_string()
        .replace(replace_path, "/auth/authorize")
        + "?"
        + uri.query().unwrap_or("")
}

pub async fn find_membership<Repo: Repository>(
    repo: &Repo,
    tenant: &TenantId,
    project: &ProjectId,
    user: &User,
) -> Result<Option<Membership>, OIDCError> {
    // Should be confirmed in AuthSessionValidationLayer from middleware/session_validation that user.tenant == tenant,
    // but double check here before querying for membership.
    if user.tenant != *tenant {
        return Err(OIDCError::new(
            OIDCErrorCode::AccessDenied,
            Some("User does not belong to the tenant.".to_string()),
            None,
        ));
    }

    match &user.role {
        UserRole::Owner | UserRole::Admin => Ok(None),
        UserRole::Member => {
            // Check that user is a member of the tenant.
            let membership = ProjectModelAdmin::search(
                repo,
                tenant,
                project,
                &MembershipSearchClaims {
                    user_id: Some(UserId::new(user.id.clone())),
                    role: None,
                },
            )
            .await
            .map_err(|_e| {
                OIDCError::new(
                    OIDCErrorCode::ServerError,
                    Some("Failed to search for user membership.".to_string()),
                    None,
                )
            })?;

            Ok(membership.into_iter().next())
        }
    }
}

#[derive(TypedPath)]
#[typed_path("/authorize")]
pub struct AuthorizePath;

pub async fn authorize<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: AuthorizePath,
    Scopes(scopes): Scopes,
    Cached(TenantContext { tenant, branding }): Cached<TenantContext>,
    Cached(Project(project_resource)): Cached<Project>,
    Cached(ProjectIdentifier { project }): Cached<ProjectIdentifier>,
    State(app_state): State<Arc<ServerState<Repo, Search, Terminology>>>,
    OIDCClientApplication(client_app): OIDCClientApplication,
    Extension(oidc_params): Extension<OIDCParameters>,
    CSRFToken(csrf_token): CSRFToken,
    Cached(current_session): Cached<Session>,
) -> Result<Response, OIDCError> {
    let state = oidc_params.parameters.get("state").ok_or_else(|| {
        OIDCError::new(
            OIDCErrorCode::InvalidRequest,
            Some("state parameter is required.".to_string()),
            None,
        )
    })?;

    let response_type = oidc_params.parameters.get("response_type").ok_or_else(|| {
        OIDCError::new(
            OIDCErrorCode::InvalidRequest,
            Some("response_type parameter is required.".to_string()),
            None,
        )
    })?;

    let redirect_uri = oidc_params.parameters.get("redirect_uri").ok_or_else(|| {
        OIDCError::new(
            OIDCErrorCode::InvalidRequest,
            Some("redirect_uri parameter is required.".to_string()),
            None,
        )
    })?;

    // RFC 6749 §4.1.2.1: a redirect URI that is not registered for the client
    // must not be redirected to, not even to report the error. Doing so would
    // make this endpoint an open redirector, so these two errors are answered
    // in the response instead of carrying the URI.
    if !is_valid_redirect_url(redirect_uri, &client_app) {
        return Err(OIDCError::new(
            OIDCErrorCode::InvalidRequest,
            Some("Invalid redirect URI.".to_string()),
            None,
        ));
    }

    let uri = Uri::try_from(redirect_uri).map_err(|_| {
        OIDCError::new(
            OIDCErrorCode::InvalidRequest,
            Some("Invalid redirect URI.".to_string()),
            None,
        )
    })?;

    // SMART App Launch: `aud` names the FHIR server the app will send the
    // token to. Tokens from here only work at this project's FHIR endpoint, so
    // any other value means the app was pointed at a different server.
    if let Some(aud) = oidc_params.parameters.get("aud") {
        let fhir_root =
            api_fhir_root_url(&app_state.config.api_uri, &tenant, &project).map_err(|_| {
                OIDCError::new(
                    OIDCErrorCode::ServerError,
                    Some("Failed to derive the FHIR base URL.".to_string()),
                    Some(redirect_uri.to_string()),
                )
            })?;

        if !is_project_fhir_base(aud, &fhir_root) {
            return Err(OIDCError::new(
                OIDCErrorCode::InvalidRequest,
                Some("aud must be this project's FHIR base URL.".to_string()),
                Some(redirect_uri.to_string()),
            ));
        }
    }

    let completed_auth_state = session::user::get_completed_authorization_state(&current_session)
        .await
        .map_err(|_e| {
            OIDCError::new(
                OIDCErrorCode::ServerError,
                Some("Failed to retrieve user from session.".to_string()),
                Some(redirect_uri.to_string()),
            )
        })
        .map_err(|_| {
            OIDCError::new(
                OIDCErrorCode::AccessDenied,
                Some("User is not authenticated.".to_string()),
                Some(redirect_uri.to_string()),
            )
        })?;

    // Verify the user has access to the given project.

    let membership = find_membership(
        &*app_state.repo,
        &tenant,
        &project,
        &completed_auth_state.user,
    )
    .await?;

    if membership.is_none() && completed_auth_state.user.role == UserRole::Member {
        warn!(
            "User '{}' is not a member of project '{}'",
            completed_auth_state.user.id, project
        );
        session::user::clear_authorization_state(&current_session)
            .await
            .map_err(|_e| {
                OIDCError::new(
                    OIDCErrorCode::ServerError,
                    Some("Failed to clear user session.".to_string()),
                    Some(redirect_uri.to_string()),
                )
            })?;

        return Err(OIDCError::new(
            OIDCErrorCode::AccessDenied,
            Some(format!(
                "User is not a member of project '{}'.",
                project_resource
                    .name
                    .value
                    .as_deref()
                    .unwrap_or(project.as_ref())
            )),
            Some(redirect_uri.to_string()),
        ));
    }

    let Some(code_challenge) = oidc_params.parameters.get("code_challenge") else {
        return Err(OIDCError::new(
            OIDCErrorCode::InvalidRequest,
            Some("code_challenge parameter is required.".to_string()),
            Some(redirect_uri.to_string()),
        ));
    };

    let Some(code_challenge_method) = oidc_params
        .parameters
        .get("code_challenge_method")
        .and_then(|code_challenge_method| {
            PKCECodeChallengeMethod::try_from(code_challenge_method.as_str()).ok()
        })
    else {
        return Err(OIDCError::new(
            OIDCErrorCode::InvalidRequest,
            Some("code_challenge_method must be a valid PKCE code challenge method.".to_string()),
            Some(redirect_uri.to_string()),
        ));
    };

    let client_id = client_app.id.clone().ok_or_else(|| {
        OIDCError::new(
            OIDCErrorCode::ServerError,
            Some("Failed to retrieve client ID.".to_string()),
            Some(redirect_uri.to_string()),
        )
    })?;

    // SMART EHR launch. Checked before consent, so nobody is asked to approve
    // an app for a launch that cannot be used. Without the `launch` scope a
    // launch value is ignored; with the scope but no value there is simply no
    // context, and the scope is not granted when the token is issued.
    let launch_context = match oidc_params.parameters.get("launch") {
        Some(launch)
            if scopes
                .contains_scope(&Scope::SMART(SmartScope::LaunchSystem(LaunchSystemScope))) =>
        {
            Some(
                resolve_launch(
                    &*app_state.repo,
                    &tenant,
                    &project,
                    &client_id,
                    &completed_auth_state.user.id,
                    launch,
                    redirect_uri,
                )
                .await?,
            )
        }
        _ => None,
    };

    let existing_scopes = ProjectModelAdmin::<CreateScope, _, _, _, _>::read(
        &*app_state.repo,
        &tenant,
        &project,
        &ScopeKey::new(
            ClientId::new(client_id.to_string()),
            UserId::new(completed_auth_state.user.id.clone()),
        ),
    )
    .await
    .map_err(|_e| {
        OIDCError::new(
            OIDCErrorCode::ServerError,
            Some("Failed to retrieve users accepted scopes.".to_string()),
            Some(redirect_uri.to_string()),
        )
    })?;

    if existing_scopes.as_ref().map(|s| &s.scope) != Some(&scopes) {
        let client_scope = client_app
            .scope
            .as_ref()
            .and_then(|s| s.value.clone())
            .unwrap_or_default();
        // A malformed client scope must not panic the endpoint (old
        // `Scopes::from` used `.expect`).
        let allowed = haste_jwt::scopes::Scopes::try_from(client_scope.as_str()).map_err(|_| {
            OIDCError::new(
                OIDCErrorCode::InvalidScope,
                Some("Client application has an invalid scope configured.".to_string()),
                Some(redirect_uri.to_string()),
            )
        })?;
        verify_requested_scope_is_subset(&scopes, &allowed)?;

        return Ok(pages::scope_approval::scope_approval_html(
            &tenant,
            &project_resource,
            &client_app,
            &ScopeForm {
                csrf_token,
                client_id: client_app
                    .id
                    .as_ref()
                    .map(|s| s.to_string())
                    .unwrap_or("".to_string()),
                scope: scopes,
                response_type: response_type.clone(),
                state: state.clone(),
                code_challenge: code_challenge.to_string(),
                code_challenge_method: String::from(code_challenge_method),
                redirect_uri: redirect_uri.to_string(),
                launch: oidc_params.parameters.get("launch").cloned(),
                aud: oidc_params.parameters.get("aud").cloned(),
                accept: None,
            },
            Some(&branding),
        )
        .into_response());
    }

    let authorization_code = ProjectModelAdmin::create(
        &*app_state.repo,
        &tenant,
        &project,
        CreateAuthorizationCode {
            membership: membership.as_ref().map(|m| m.resource_id.clone()),
            expires_in: Duration::from_secs(60 * 5), // 5 minutes
            kind: AuthorizationCodeKind::OAuth2CodeGrant,
            user_id: completed_auth_state.user.id,
            client_id: Some(client_id.to_string()),
            pkce_code_challenge: Some(code_challenge.to_string()),
            pkce_code_challenge_method: Some(code_challenge_method),
            redirect_uri: Some(redirect_uri.to_string()),
            // The token endpoint reads the launch's patient and encounter here.
            meta: launch_context.map(|context| Json(json!({ "launch": context }))),
        },
    )
    .await
    .map_err(|_e| {
        OIDCError::new(
            OIDCErrorCode::ServerError,
            Some("Failed to create authorization code.".to_string()),
            Some(redirect_uri.to_string()),
        )
    })?;

    // Percent-encode the parameters and avoid unwraps: a `state` containing a
    // space, `+`, `&` or `#` previously either did not round-trip or panicked
    // the builder (a 500 via CatchPanicLayer).
    let query = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("code", authorization_code.code.as_str())
        .append_pair("state", state.as_str())
        .finish();
    let authority = uri.authority().cloned().ok_or_else(|| {
        OIDCError::new(
            OIDCErrorCode::ServerError,
            Some("Request URI has no authority.".to_string()),
            None,
        )
    })?;
    let redirection = Uri::builder()
        .scheme(uri.scheme().cloned().unwrap_or(Scheme::HTTPS))
        .authority(authority)
        .path_and_query(format!("{}?{}", uri.path(), query))
        .build()
        .map_err(|_| {
            OIDCError::new(
                OIDCErrorCode::ServerError,
                Some("Failed to build redirect.".to_string()),
                None,
            )
        })?;

    let redirect = Redirect::to(&redirection.to_string());
    let response = redirect.into_response();
    Ok(response)
}
