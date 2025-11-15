use crate::services::AppState;
use axum::{
    extract::{FromRequestParts, Path},
    http::request::Parts,
    response::{IntoResponse, Response},
};
use haste_fhir_model::r4::generated::resources::{Resource, ResourceType};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, ResourceId, TenantId};
use haste_repository::Repository;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Clone)]
pub struct Project(pub haste_fhir_model::r4::generated::resources::Project);

impl<Repo, Search, Terminology> FromRequestParts<Arc<AppState<Repo, Search, Terminology>>>
    for Project
where
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState<Repo, Search, Terminology>>,
    ) -> Result<Self, Self::Rejection> {
        let TenantIdentifier { tenant } = TenantIdentifier::from_request_parts(parts, state)
            .await
            .map_err(|err| err.into_response())?;

        let ProjectIdentifier { project } = ProjectIdentifier::from_request_parts(parts, state)
            .await
            .map_err(|err| err.into_response())?;

        let project_resource = state
            .repo
            .read_latest(
                &tenant,
                &ProjectId::System,
                &ResourceType::Project,
                &ResourceId::new(project.as_ref().to_string()),
            )
            .await
            .map_err(|err| err.into_response())?;

        if let Some(resource) = project_resource
            && let Resource::Project(project) = resource
        {
            Ok(Self(project))
        } else {
            Err(OperationOutcomeError::fatal(
                haste_fhir_model::r4::generated::terminology::IssueType::NotFound(None),
                format!(
                    "Project resource '{}' not found for tenant '{}'",
                    project.as_ref(),
                    tenant.as_ref()
                ),
            )
            .into_response())
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct ProjectIdentifier {
    pub project: ProjectId,
}

impl<S: Send + Sync> FromRequestParts<S> for ProjectIdentifier {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(project) = Path::<ProjectIdentifier>::from_request_parts(parts, state)
            .await
            .map_err(|err| err.into_response())?;

        Ok(project)
    }
}

#[derive(Deserialize, Clone)]
pub struct TenantIdentifier {
    pub tenant: TenantId,
}

impl<S: Send + Sync> FromRequestParts<S> for TenantIdentifier {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(tenant_information) = Path::<TenantIdentifier>::from_request_parts(parts, state)
            .await
            .map_err(|err| err.into_response())?;

        Ok(tenant_information)
    }
}
