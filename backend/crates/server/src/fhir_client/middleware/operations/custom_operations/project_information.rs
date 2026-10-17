use crate::fhir_client::middleware::{Backend, operations::ServerOperationContext};
use haste_fhir_client::request::InvocationRequest;
use haste_fhir_generated_ops::generated::ProjectInformation;
use haste_fhir_model::r4::generated::{
    resources::{Resource, ResourceType},
    terminology::IssueType,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_ops::OperationExecutor;
use haste_jwt::{ProjectId, ResourceId, TenantId};
use haste_repository::fhir::FHIRRepository;

pub fn project_information_op<B: Backend>() -> OperationExecutor<
    ServerOperationContext<B>,
    ProjectInformation::Input,
    ProjectInformation::Output,
> {
    OperationExecutor::new(
        &ProjectInformation::OPERATION_DEFINITION,
        Box::new(
            |context: ServerOperationContext<B>,
             tenant: TenantId,
             project: ProjectId,
             _request: &InvocationRequest,
             _input: ProjectInformation::Input| {
                Box::pin(async move {
                    let output = context
                        .state
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
                        Err(OperationOutcomeError::fatal(
                            IssueType::not_found(),
                            "Project not found".to_string(),
                        ))
                    }
                })
            },
        ),
    )
}
