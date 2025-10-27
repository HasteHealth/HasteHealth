use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use axum_extra::extract::Cached;
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_repository::{Repository, admin::TenantAuthAdmin, types::project::CreateProject};
use std::sync::Arc;

use crate::{
    extract::path_tenant::{ProjectIdentifier, TenantIdentifier},
    services::AppState,
};

pub async fn project_exists<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    State(state): State<Arc<AppState<Repo, Search, Terminology>>>,
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    Cached(ProjectIdentifier { project }): Cached<ProjectIdentifier>,
    request: Request,
    next: Next,
) -> Result<Response, OperationOutcomeError> {
    // If not found automatically will error.
    TenantAuthAdmin::<CreateProject, _, _, _, _>::read(
        &*state.repo,
        &tenant,
        &project.as_ref().to_string(),
    )
    .await
    .map_err(|_| {
        OperationOutcomeError::fatal(
            oxidized_fhir_model::r4::generated::terminology::IssueType::NotFound(None),
            format!(
                "Project '{}' not found for tenant '{}'",
                project.as_ref(),
                tenant.as_ref()
            ),
        )
    })?;

    Ok(next.run(request).await)
}
