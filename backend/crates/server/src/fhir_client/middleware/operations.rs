use crate::fhir_client::middleware::{
    ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput, ServerMiddlewareState,
};
use oxidized_fhir_client::request::{FHIRInvokeSystemResponse, FHIRRequest, FHIRResponse};
use oxidized_fhir_generated_ops::generated::ValueSetExpand;
use oxidized_fhir_model::r4::generated::{resources::Resource, terminology::IssueType};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_ops::{OperationExecutor, OperationInvocation, Param};
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_repository::Repository;
use std::sync::Arc;

/// Sets tenant to search in system for artifact resources IE SDs etc..
pub fn operations<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    state: ServerMiddlewareState<Repo, Search, Terminology>,
    mut context: ServerMiddlewareContext,
    _next: Option<Arc<ServerMiddlewareNext<Repo, Search, Terminology>>>,
) -> ServerMiddlewareOutput {
    Box::pin(async move {
        let op_executor: OperationExecutor<
            Arc<Terminology>,
            ValueSetExpand::Input,
            ValueSetExpand::Output,
        > = OperationExecutor::new(Box::new(
            |ctx: Arc<Terminology>, input: ValueSetExpand::Input| {
                Box::pin(async move {
                    // Implement the logic for the $expand operation here.
                    // For demonstration, we'll just return an error indicating it's not implemented.
                    let output = ctx.expand(input).await?;
                    Ok(output)
                })
            },
        ));

        let output: Resource = match &context.request {
            FHIRRequest::InvokeInstance(instance_request) => {
                let output = op_executor
                    .execute(
                        state.terminology.clone(),
                        Param::Parameters(instance_request.parameters.clone()),
                    )
                    .await?;
                Ok(Resource::from(output))
            }
            FHIRRequest::InvokeType(type_request) => {
                let output = op_executor
                    .execute(
                        state.terminology.clone(),
                        Param::Parameters(type_request.parameters.clone()),
                    )
                    .await?;
                Ok(Resource::from(output))
            }
            FHIRRequest::InvokeSystem(system_request) => {
                let output = op_executor
                    .execute(
                        state.terminology.clone(),
                        Param::Parameters(system_request.parameters.clone()),
                    )
                    .await?;
                Ok(Resource::from(output))
            }
            _ => Err(OperationOutcomeError::fatal(
                IssueType::Exception(None),
                "Operation not supported".to_string(),
            )),
        }?;

        context.response = Some(FHIRResponse::InvokeSystem(FHIRInvokeSystemResponse {
            resource: output,
        }));

        Ok(context)
    })
}
