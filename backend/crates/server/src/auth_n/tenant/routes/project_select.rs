use crate::{
    auth_n::oidc::hardcoded_clients::admin_app, extract::path_tenant::TenantIdentifier,
    fhir_client::ServerCTX, services::AppState, ui::pages,
};
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::{extract::Cached, routing::TypedPath};
use haste_fhir_client::{FHIRClient, url::ParsedParameters};
use haste_fhir_model::r4::generated::resources::{Resource, ResourceType};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::ProjectId;
use haste_repository::Repository;
use std::sync::Arc;

#[derive(TypedPath)]
#[typed_path("/project-select")]
pub struct ProjectSelect;

pub async fn project_get<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: ProjectSelect,
    State(state): State<Arc<AppState<Repo, Search, Terminology>>>,
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
) -> Result<Response, OperationOutcomeError> {
    let tenant_projects = state
        .fhir_client
        .search_type(
            Arc::new(ServerCTX::system(
                tenant.clone(),
                ProjectId::System,
                state.fhir_client.clone(),
                state.rate_limit.clone(),
            )),
            ResourceType::Project,
            ParsedParameters::new(vec![]),
        )
        .await?
        .entry
        .unwrap_or(vec![])
        .into_iter()
        .filter_map(|e| e.resource)
        .filter_map(|r| match *r {
            Resource::Project(project) => Some(project),
            _ => None,
        })
        .collect::<Vec<_>>();

    // If there is only one project, skip the project select page and go directly to the login page for that project
    if tenant_projects.len() == 1
        && let Some(project) = tenant_projects.first()
        && let Some(project_id) = project.id.as_ref().map(|id| ProjectId::new(id.clone()))
        && let Some(admin_app_url) =
            admin_app::redirect_url(state.config.as_ref(), &tenant, &project_id)
    {
        Ok(Redirect::to(&admin_app_url).into_response())
    } else {
        let response = pages::project_select::project_select_html(
            state.config.as_ref(),
            &tenant,
            &tenant_projects,
        )?;

        Ok(response.into_response())
    }
}
