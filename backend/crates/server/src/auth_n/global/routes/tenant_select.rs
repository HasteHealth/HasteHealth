use crate::auth_n::oidc::hardcoded_clients::admin_app;
use crate::{services::AppState, ui::pages::tenant_select::tenant_select_form_html};
use axum::response::{IntoResponse, Redirect, Response};
use axum::{Form, extract::State};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, TenantId};
use haste_repository::Repository;
use std::sync::Arc;

#[derive(serde::Deserialize, axum_extra::routing::TypedPath)]
#[typed_path("/tenant-select")]
pub struct TenantSelectGet {}

pub async fn tenant_select_get<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: TenantSelectGet,
    State(_app_state): State<Arc<AppState<Repo, Search, Terminology>>>,
) -> Result<Response, OperationOutcomeError> {
    Ok(tenant_select_form_html("/global/tenant-select", None).into_response())
}

#[derive(serde::Deserialize)]
pub struct TenantSelectForm {
    pub tenant: String,
    pub project: Option<String>,
}

#[derive(serde::Deserialize, axum_extra::routing::TypedPath)]
#[typed_path("/tenant-select")]
pub struct TenantSelectPost {}

pub async fn tenant_select_post<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: TenantSelectPost,
    State(app_state): State<Arc<AppState<Repo, Search, Terminology>>>,
    Form(form): Form<TenantSelectForm>,
) -> Result<Response, OperationOutcomeError> {
    let tenant_id = TenantId::new(form.tenant);
    let project_id = if let Some(project) = form.project {
        ProjectId::new(project)
    } else {
        ProjectId::System
    };

    let admin_app_redirect_url =
        admin_app::redirect_url(app_state.config.as_ref(), &tenant_id, &project_id);

    if let Some(admin_app_redirect_url) = admin_app_redirect_url.as_ref() {
        Ok(Redirect::to(&admin_app_redirect_url).into_response())
    } else {
        tracing::error!(
            "Failed to get admin app redirect URL for tenant '{}' and project '{}'",
            tenant_id.as_ref(),
            project_id.as_ref()
        );
        Err(OperationOutcomeError::error(
            haste_fhir_model::r4::generated::terminology::IssueType::Exception(None),
            "Failed to determine admin app URL for tenant".to_string(),
        ))
    }
}
