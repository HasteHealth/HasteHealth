use crate::{
    // auth_n::oidc::extract::client_app::OIDCClientApplication,
    // extract::path_tenant::{Project, Tenant},
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
use oxidized_repository::Repository;
use serde::Deserialize;
use std::sync::Arc;
use tower_sessions::Session;

#[derive(TypedPath)]
#[typed_path("/scopes")]
pub struct ScopePost;

#[derive(Deserialize, Debug)]
pub struct ScopeForm {
    scope: String,
}

pub async fn scope_post<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: ScopePost,
    _uri: OriginalUri,
    State(_state): State<Arc<AppState<Repo, Search, Terminology>>>,
    _current_session: Session,
    Form(scope_data): Form<ScopeForm>,
    // OIDCClientApplication(_client_app): OIDCClientApplication,
    // Tenant { tenant }: Tenant,
    // Project { project }: Project,
) -> Result<Response, OperationOutcomeError> {
    println!("Scope data: {:?}", scope_data.scope);
    Err(OperationOutcomeError::error(
        IssueType::NotSupported(None),
        "Not implemented.".to_string(),
    ))
}
