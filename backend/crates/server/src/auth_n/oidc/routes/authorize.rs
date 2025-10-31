use crate::{
    auth_n::{
        oidc::{
            extract::{client_app::OIDCClientApplication, scopes::Scopes},
            middleware::OIDCParameters,
            routes::scope::{ScopeForm, verify_requested_scope_is_subset},
            utilities::is_valid_redirect_url,
        },
        session,
    },
    extract::path_tenant::{Project, ProjectIdentifier, TenantIdentifier},
    services::AppState,
    ui::pages,
};
use axum::{
    Extension,
    extract::State,
    http::{Uri, uri::Scheme},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::{extract::Cached, routing::TypedPath};
use oxidized_fhir_model::r4::generated::terminology::IssueType;
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_repository::{
    Repository,
    admin::ProjectAuthAdmin,
    types::{
        ProjectId, TenantId,
        authorization_code::{
            AuthorizationCodeKind, CreateAuthorizationCode, PKCECodeChallengeMethod,
        },
        membership::{Membership, MembershipSearchClaims},
        scope::{ClientId, CreateScope, ScopeKey, UserId},
        user::{User, UserRole},
    },
};
use std::{sync::Arc, time::Duration};
use tower_sessions::Session;

pub async fn find_membership<Repo: Repository>(
    repo: &Repo,
    tenant: &TenantId,
    project: &ProjectId,
    user: &User,
) -> Result<Option<Membership>, OperationOutcomeError> {
    match &user.role {
        UserRole::Owner | UserRole::Admin => Ok(None),
        UserRole::Member => {
            // Check that user is a member of the tenant.
            let membership = ProjectAuthAdmin::search(
                repo,
                &tenant,
                &project,
                &MembershipSearchClaims {
                    user_id: Some(UserId::new(user.id.clone())),
                    role: None,
                },
            )
            .await?;

            Ok(membership.into_iter().next())
        }
    }
}

#[derive(TypedPath)]
#[typed_path("/authorize")]
pub struct Authorize;

pub async fn authorize<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: Authorize,
    Scopes(scopes): Scopes,
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    Cached(Project(project_resource)): Cached<Project>,
    Cached(ProjectIdentifier { project }): Cached<ProjectIdentifier>,
    State(app_state): State<Arc<AppState<Repo, Search, Terminology>>>,
    OIDCClientApplication(client_app): OIDCClientApplication,
    Extension(oidc_params): Extension<OIDCParameters>,
    Cached(current_session): Cached<Session>,
) -> Result<Response, OperationOutcomeError> {
    let user = session::user::get_user(&current_session, &tenant)
        .await?
        .unwrap();
    // Verify the user has access to the given project.

    let membership = find_membership(&*app_state.repo, &tenant, &project, &user).await?;

    if membership.is_none() && &user.role == &UserRole::Member {
        session::user::clear_user(&current_session, &tenant).await?;
        return Err(OperationOutcomeError::error(
            IssueType::Forbidden(None),
            format!(
                "User is not a member of project '{}'.",
                project_resource
                    .name
                    .as_ref()
                    .and_then(|n| n.value.as_ref())
                    .map(|s| s.as_str())
                    .unwrap_or(project.as_ref())
            ),
        ));
    }

    let state = oidc_params.parameters.get("state").ok_or_else(|| {
        OperationOutcomeError::error(
            IssueType::Invalid(None),
            "state parameter is required.".to_string(),
        )
    })?;

    let response_type = oidc_params.parameters.get("response_type").ok_or_else(|| {
        OperationOutcomeError::error(
            IssueType::Invalid(None),
            "response_type parameter is required.".to_string(),
        )
    })?;

    let redirect_uri = oidc_params.parameters.get("redirect_uri").ok_or_else(|| {
        OperationOutcomeError::error(
            IssueType::Invalid(None),
            "redirect_uri parameter is required.".to_string(),
        )
    })?;

    if !is_valid_redirect_url(&redirect_uri, &client_app) {
        return Err(OperationOutcomeError::error(
            IssueType::Invalid(None),
            "Invalid redirect URI.".to_string(),
        ));
    }

    let uri = Uri::try_from(redirect_uri).map_err(|_| {
        OperationOutcomeError::error(IssueType::Invalid(None), "Invalid redirect uri".to_string())
    })?;

    let Some(code_challenge) = oidc_params.parameters.get("code_challenge") else {
        return Err(OperationOutcomeError::error(
            IssueType::Invalid(None),
            "code_challenge parameter is required.".to_string(),
        ));
    };

    let Some(code_challenge_method) = oidc_params
        .parameters
        .get("code_challenge_method")
        .and_then(|code_challenge_method| {
            PKCECodeChallengeMethod::try_from(code_challenge_method.as_str()).ok()
        })
    else {
        return Err(OperationOutcomeError::error(
            IssueType::Invalid(None),
            "code_challenge_method must be a valid PKCE code challenge method.".to_string(),
        ));
    };

    let client_id = client_app.id.clone().ok_or_else(|| {
        OperationOutcomeError::error(
            IssueType::Invalid(None),
            "Client ID is required.".to_string(),
        )
    })?;

    let existing_scopes = ProjectAuthAdmin::<CreateScope, _, _, _, _>::read(
        &*app_state.repo,
        &tenant,
        &project,
        &ScopeKey::new(
            ClientId::new(client_id.to_string()),
            UserId::new(user.id.clone()),
        ),
    )
    .await?;

    if existing_scopes.as_ref().map(|s| &s.scope) != Some(&scopes) {
        verify_requested_scope_is_subset(
            &scopes,
            &oxidized_jwt::scopes::Scopes::from(
                client_app
                    .scope
                    .as_ref()
                    .and_then(|s| s.value.clone())
                    .unwrap_or_default(),
            ),
        )?;

        return Ok(pages::scope_approval::scope_approval_html(
            &tenant,
            &project_resource,
            &client_app,
            &ScopeForm {
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
                accept: None,
            },
        )
        .into_response());
    }

    let authorization_code = ProjectAuthAdmin::create(
        &*app_state.repo,
        &tenant,
        &project,
        CreateAuthorizationCode {
            membership: membership.as_ref().map(|m| m.resource_id.clone()),
            expires_in: Duration::from_secs(60 * 5),
            kind: AuthorizationCodeKind::OAuth2CodeGrant,
            user_id: user.id,
            client_id: Some(client_id.to_string()),
            pkce_code_challenge: Some(code_challenge.to_string()),
            pkce_code_challenge_method: Some(code_challenge_method),
            redirect_uri: Some(redirect_uri.to_string()),
            meta: None,
        },
    )
    .await?;

    let redirection = Uri::builder()
        .scheme(uri.scheme().cloned().unwrap_or(Scheme::HTTPS))
        .authority(uri.authority().unwrap().clone())
        .path_and_query(
            uri.path().to_string() + "?code=" + &authorization_code.code + "&state=" + state,
        )
        .build()
        .unwrap();

    let redirect = Redirect::to(&redirection.to_string());
    let response = redirect.into_response();
    Ok(response)
}
