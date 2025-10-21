use crate::{
    // auth_n::oidc::extract::client_app::OIDCClientApplication,
    // extract::path_tenant::{Project, Tenant},
    auth_n::{
        oidc::{
            extract::client_app::OIDCClientApplication, routes::route_string::oidc_route_string,
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
use axum_extra::routing::TypedPath;
use oxidized_fhir_model::r4::generated::terminology::IssueType;
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_repository::{
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
    pub scope: oxidized_repository::types::scopes::Scopes,
    pub redirect_uri: String,
    pub accept: Option<String>,
}

pub async fn scope_post<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: ScopePost,
    _uri: OriginalUri,
    State(app_state): State<Arc<AppState<Repo, Search, Terminology>>>,
    current_session: Session,
    OIDCClientApplication(_client_app): OIDCClientApplication,
    TenantIdentifier { tenant }: TenantIdentifier,
    ProjectIdentifier { project }: ProjectIdentifier,
    Form(scope_data): Form<ScopeForm>,
) -> Result<Response, OperationOutcomeError> {
    let user = session::user::get_user(&current_session, &tenant)
        .await?
        .unwrap();

    if let Some("on") = scope_data.accept.as_ref().map(String::as_str) {
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
        .await?;

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
        Err(OperationOutcomeError::error(
            IssueType::Forbidden(None),
            "User did not accept the requested scopes.".to_string(),
        ))
    }
}
