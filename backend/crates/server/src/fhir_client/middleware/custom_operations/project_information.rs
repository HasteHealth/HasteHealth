use crate::fhir_client::middleware::ServerMiddlewareState;
use haste_fhir_generated_ops::generated::ProjectInformation;
use haste_fhir_model::r4::generated::{
    resources::{Resource, ResourceType},
    terminology::IssueType,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_ops::OperationExecutor;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, ResourceId, TenantId};
use haste_repository::Repository;

pub fn project_information<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>() -> OperationExecutor<
    ServerMiddlewareState<Repo, Search, Terminology>,
    ProjectInformation::Input,
    ProjectInformation::Output,
> {
    OperationExecutor::new(
        ProjectInformation::CODE.to_string(),
        Box::new(
            |ctx: ServerMiddlewareState<Repo, Search, Terminology>,
             tenant: TenantId,
             project: ProjectId,
             _input: ProjectInformation::Input| {
                Box::pin(async move {
                    let output = ctx
                        .repo
                        .read_latest(
                            &tenant,
                            &ProjectId::System,
                            &ResourceType::Project,
                            &ResourceId::new(project.to_string()),
                        )
                        .await?;

                    if let Some(resource) = output
                        && let Resource::Project(project) = resource
                    {
                        Ok(ProjectInformation::Output { project })
                    } else {
                        return Err(OperationOutcomeError::fatal(
                            IssueType::NotFound(None),
                            "Project not found".to_string(),
                        ));
                    }
                })
            },
        ),
    )
}
