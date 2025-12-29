use crate::fhir_client::middleware::operations::ServerOperationContext;
use haste_fhir_client::request::InvocationRequest;
use haste_fhir_generated_ops::generated::HasteHealthEvaluatePolicy;

use haste_fhir_model::r4::generated::{terminology::IssueType, types::Reference};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_ops::OperationExecutor;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, TenantId};
use haste_repository::Repository;

fn derive_user(user_reference: Option<Reference>) -> Result<Option<String>, OperationOutcomeError> {
    if let Some(reference_string) = user_reference
        .and_then(|u| u.reference)
        .and_then(|r| r.value)
    {
        let reference_chunks = reference_string.split('/').collect::<Vec<_>>();
        let [resource_type, resource_id] = reference_chunks.as_slice() else {
            return Err(OperationOutcomeError::error(
                IssueType::Invalid(None),
                "Invalid user reference format".to_string(),
            ));
        };

        if *resource_type != "User" {
            return Err(OperationOutcomeError::error(
                IssueType::Invalid(None),
                "User reference must refer to a User resource".to_string(),
            ));
        }

        return Ok(Some(resource_id.to_string()));
    }

    Ok(None)
}

pub fn evaluate_policy_operation<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>() -> OperationExecutor<
    ServerOperationContext<Repo, Search, Terminology>,
    HasteHealthEvaluatePolicy::Input,
    HasteHealthEvaluatePolicy::Output,
> {
    OperationExecutor::new(
        HasteHealthEvaluatePolicy::CODE.to_string(),
        Box::new(
            |context: ServerOperationContext<Repo, Search, Terminology>,
             tenant: TenantId,
             project: ProjectId,
             _request: &InvocationRequest,
             input: HasteHealthEvaluatePolicy::Input| {
                Box::pin(async move {
                    todo!();
                })
            },
        ),
    )
}
