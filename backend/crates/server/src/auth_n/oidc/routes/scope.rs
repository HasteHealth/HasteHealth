use crate::{
    // auth_n::oidc::extract::client_app::OIDCClientApplication,
    // extract::path_tenant::{Project, Tenant},
    auth_n::{
        oidc::extract::{client_app::OIDCClientApplication, scopes::Scopes},
        session,
    },
    extract::path_tenant::{Project, Tenant},
    services::AppState,
};
use axum::{
    Form,
    extract::{OriginalUri, State},
    response::Response,
};
use axum_extra::routing::TypedPath;
use oxidized_fhir_model::r4::generated::terminology::IssueType;
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_repository::{
    Repository,
    admin::ProjectAuthAdmin,
    types::scope::{ClientId, CreateScope, ScopeKey, UserId},
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
    pub scope: Scopes,
    pub redirect_uri: String,
    pub accept: Option<bool>,
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
    OIDCClientApplication(client_app): OIDCClientApplication,
    Tenant { tenant }: Tenant,
    Project { project }: Project,
    Form(scope_data): Form<ScopeForm>,
) -> Result<Response, OperationOutcomeError> {
    let user = session::user::get_user(&current_session).await?.unwrap();

    if scope_data.accept.unwrap_or(false) {
        let scopes = ProjectAuthAdmin::create(
            &*app_state.repo,
            &tenant,
            &project,
            CreateScope {
                client: ClientId::new(scope_data.client_id),
                user_: UserId::new(user.id),
                scope: String::from(scope_data.scope),
            },
        )
        .await?;
    }

    Err(OperationOutcomeError::error(
        IssueType::NotSupported(None),
        "Not implemented.".to_string(),
    ))
}
