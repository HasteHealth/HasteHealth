use crate::fhir_client::{
    ServerCTX,
    middleware::{
        ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput,
        ServerMiddlewareState,
    },
};
use oxidized_fhir_client::{
    middleware::MiddlewareChain,
    request::{FHIRInvokeSystemResponse, FHIRRequest, FHIRResponse},
};
use oxidized_fhir_generated_ops::generated::ValueSetExpand;
use oxidized_fhir_model::r4::generated::{resources::Resource, terminology::IssueType};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_ops::{OperationExecutor, OperationInvocation, Param};
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_repository::Repository;
use std::sync::Arc;

pub struct OperationMiddleware<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
> {
    operations: Arc<
        Vec<
            OperationExecutor<
                ServerMiddlewareState<Repo, Search, Terminology>,
                ValueSetExpand::Input,
                ValueSetExpand::Output,
            >,
        >,
    >,
}

impl<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
> OperationMiddleware<Repo, Search, Terminology>
{
    pub fn new() -> Self {
        let op_executor: OperationExecutor<
            ServerMiddlewareState<Repo, Search, Terminology>,
            ValueSetExpand::Input,
            ValueSetExpand::Output,
        > = OperationExecutor::new(Box::new(
            |ctx: ServerMiddlewareState<Repo, Search, Terminology>,
             input: ValueSetExpand::Input| {
                Box::pin(async move {
                    // Implement the logic for the $expand operation here.
                    // For demonstration, we'll just return an error indicating it's not implemented.
                    let output = ctx.terminology.expand(input).await?;
                    Ok(output)
                })
            },
        ));

        OperationMiddleware {
            operations: Arc::new(vec![op_executor]),
        }
    }
}

impl<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>
    MiddlewareChain<
        ServerMiddlewareState<Repo, Search, Terminology>,
        Arc<ServerCTX>,
        FHIRRequest,
        FHIRResponse,
        OperationOutcomeError,
    > for OperationMiddleware<Repo, Search, Terminology>
{
    /// Sets tenant to search in system for artifact resources IE SDs etc..
    fn call(
        &self,
        state: ServerMiddlewareState<Repo, Search, Terminology>,
        mut context: ServerMiddlewareContext,
        _next: Option<Arc<ServerMiddlewareNext<Repo, Search, Terminology>>>,
    ) -> ServerMiddlewareOutput {
        let executors = self.operations.clone();
        Box::pin(async move {
            let op_executor = &executors[0];
            let output: Resource = match &context.request {
                FHIRRequest::InvokeInstance(instance_request) => {
                    let output = op_executor
                        .execute(
                            state,
                            Param::Parameters(instance_request.parameters.clone()),
                        )
                        .await?;
                    Ok(Resource::from(output))
                }
                FHIRRequest::InvokeType(type_request) => {
                    let output = op_executor
                        .execute(state, Param::Parameters(type_request.parameters.clone()))
                        .await?;
                    Ok(Resource::from(output))
                }
                FHIRRequest::InvokeSystem(system_request) => {
                    let output = op_executor
                        .execute(state, Param::Parameters(system_request.parameters.clone()))
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
}
