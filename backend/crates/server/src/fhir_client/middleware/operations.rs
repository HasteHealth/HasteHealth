use crate::fhir_client::middleware::{
    ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput, ServerMiddlewareState,
};
use oxidized_fhir_client::request::{FHIRInvokeSystemResponse, FHIRRequest, FHIRResponse};
use oxidized_fhir_generated_ops::generated::ValueSetExpand;
use oxidized_fhir_model::r4::generated::{resources::ValueSet, terminology::IssueType};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_ops::{OperationExecutor, OperationInvocation, Param};
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_repository::Repository;
use std::sync::{Arc, LazyLock};

static VALUESET_EXPAND_OPERATION: LazyLock<
    OperationExecutor<ValueSetExpand::Input, ValueSetExpand::Output>,
> = LazyLock::new(|| {
    OperationExecutor::new(Box::new(|_input| {
        Box::pin(async move {
            // Implement the logic for the $expand operation here.
            // For demonstration, we'll just return an error indicating it's not implemented.
            Ok(ValueSetExpand::Output {
                return_: ValueSet {
                    ..Default::default()
                },
            })
        })
    }))
});

/// Sets tenant to search in system for artifact resources IE SDs etc..
pub fn operations<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    state: ServerMiddlewareState<Repo, Search, Terminology>,
    mut context: ServerMiddlewareContext,
    next: Option<Arc<ServerMiddlewareNext<Repo, Search, Terminology>>>,
) -> ServerMiddlewareOutput {
    Box::pin(async move {
        let ctx = context.ctx.clone();

        let output: Param<_> = match &context.request {
            FHIRRequest::InvokeInstance(instance_request) => {
                let p = &*VALUESET_EXPAND_OPERATION;
                let output = p
                    .execute(Param::Parameters(instance_request.parameters.clone()))
                    .await?;
                Ok(Param::Value(output))
            }
            FHIRRequest::InvokeType(type_request) => {
                let p = &*VALUESET_EXPAND_OPERATION;
                let output = p
                    .execute(Param::Parameters(type_request.parameters.clone()))
                    .await?;
                Ok(Param::Value(output))
            }
            FHIRRequest::InvokeSystem(system_request) => {
                let p = &*VALUESET_EXPAND_OPERATION;
                let output = p
                    .execute(Param::Parameters(system_request.parameters.clone()))
                    .await?;
                Ok(Param::Value(output))
            }
            _ => Err(OperationOutcomeError::fatal(
                IssueType::Exception(None),
                "Operation not supported".to_string(),
            )),
        }?;

        context.response = Some(FHIRResponse::InvokeSystem(FHIRInvokeSystemResponse {
            resource: output.as_parameters(),
        }));

        Ok(context)
    })
}
