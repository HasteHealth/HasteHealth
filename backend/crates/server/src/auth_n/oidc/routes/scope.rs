use crate::{
    // auth_n::oidc::extract::client_app::OIDCClientApplication,
    // extract::path_tenant::{Project, Tenant},
    auth_n::{
        oidc::{
            error::{OIDCError, OIDCErrorCode},
            extract::client_app::OIDCClientApplication,
            routes::route_string::oidc_route_string,
        },
        session,
    },
    extract::path_tenant::{ProjectIdentifier, TenantIdentifier},
    services::AppState,
};
use axum::{
    Form,
    extract::{OriginalUri, State},
    response::{IntoResponse, Response},
};
use axum_extra::{extract::Cached, routing::TypedPath};
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::scopes::Scopes;
use haste_repository::{
    Repository,
    admin::ProjectAuthAdmin,
    types::scope::{ClientId, CreateScope, UserId},
};
use serde::Deserialize;
use std::sync::Arc;
use tower_sessions::Session;

#[derive(TypedPath)]
#[typed_path("/scope")]
pub struct ScopePost;

#[derive(Deserialize, Debug)]
pub struct ScopeForm {
    pub client_id: String,
    pub response_type: String,
    pub state: String,
    pub code_challenge: String,
    pub code_challenge_method: String,
    pub scope: haste_jwt::scopes::Scopes,
    pub redirect_uri: String,
    pub accept: Option<String>,
}

pub fn verify_requested_scope_is_subset(
    requested: &Scopes,
    allowed: &Scopes,
) -> Result<(), OIDCError> {
    for scope in requested.0.iter() {
        if !allowed.0.contains(scope) {
            return Err(OIDCError::new(
                OIDCErrorCode::InvalidScope, 
                Some("Requested scope '{}' is not allowed. Check client configuration for what scopes are allowed.".to_string()),
                 None
                )
            );
        }
    }
    Ok(())
}

pub async fn scope_post<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: ScopePost,
    _uri: OriginalUri,
    State(app_state): State<Arc<AppState<Repo, Search, Terminology>>>,
    Cached(current_session): Cached<Session>,
    OIDCClientApplication(client_app): OIDCClientApplication,
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    Cached(ProjectIdentifier { project }): Cached<ProjectIdentifier>,
    Form(scope_data): Form<ScopeForm>,
) -> Result<Response, OIDCError> {
    let user = session::user::get_user(&current_session, &tenant)
        .await.map_err(|_| {
            OIDCError::new(
                OIDCErrorCode::ServerError,
                Some("Failed to retrieve user from session.".to_string()),
                Some(scope_data.redirect_uri.clone()),
            )
        })?
        .unwrap();

    if let Some("on") = scope_data.accept.as_ref().map(String::as_str) {
        verify_requested_scope_is_subset(
            &scope_data.scope,
            &Scopes::from(
                client_app
                    .scope
                    .as_ref()
                    .and_then(|s| s.value.clone())
                    .unwrap_or_default(),
            ),
        )?;

        ProjectAuthAdmin::create(
            &*app_state.repo,
            &tenant,
            &project,
            CreateScope {
                client: ClientId::new(scope_data.client_id.clone()),
                user_: UserId::new(user.id),
                scope: scope_data.scope.clone(),
            },
        )
        .await.map_err(|_| {
            OIDCError::new(
                OIDCErrorCode::ServerError,
                Some("Failed to create scope authorization.".to_string()),
                Some(scope_data.redirect_uri.clone()),
            )
        })?;

        let authorization_route = oidc_route_string(&tenant, &project, "auth/authorize")
            .to_str()
            .expect("Could not create authorize route.")
            .to_string()
            + "?client_id="
            + &scope_data.client_id
            + "&response_type="
            + &scope_data.response_type
            + "&state="
            + &scope_data.state
            + "&code_challenge="
            + &scope_data.code_challenge
            + "&code_challenge_method="
            + &scope_data.code_challenge_method
            + "&scope="
            + &String::from(scope_data.scope)
            + "&redirect_uri="
            + &scope_data.redirect_uri;
        let redirect = axum::response::Redirect::to(&authorization_route);
        Ok(redirect.into_response())
    } else {
        Err(OIDCError::new(
            OIDCErrorCode::AccessDenied,
            Some("User did not accept the requested scopes.".to_string()),
            Some(scope_data.redirect_uri)
        ))
    }
}
