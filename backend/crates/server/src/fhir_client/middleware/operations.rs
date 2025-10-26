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
use oxidized_fhir_model::r4::generated::{
    resources::{Parameters, Resource},
    terminology::IssueType,
};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_ops::{OperationExecutor, OperationInvocation, Param};
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_repository::Repository;
use std::{pin::Pin, sync::Arc};

struct ServerOperations<CTX>(Arc<Vec<Box<dyn OperationInvocation<CTX>>>>);

impl<CTX> Clone for ServerOperations<CTX> {
    fn clone(&self) -> Self {
        ServerOperations(self.0.clone())
    }
}

impl<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
> ServerOperations<ServerMiddlewareState<Repo, Search, Terminology>>
{
    pub fn new() -> Self {
        let executors: Vec<
            Box<dyn OperationInvocation<ServerMiddlewareState<Repo, Search, Terminology>>>,
        > = vec![Box::new(OperationExecutor::new(
            ValueSetExpand::CODE.to_string(),
            Box::new(
                |ctx: ServerMiddlewareState<Repo, Search, Terminology>,
                 input: ValueSetExpand::Input| {
                    Box::pin(async move {
                        let output = ctx.terminology.expand(input).await?;
                        Ok(output)
                    })
                },
            ),
        ))];

        Self(Arc::new(executors))
    }

    pub fn find_operation(
        &self,
        code: &str,
    ) -> Option<&dyn OperationInvocation<ServerMiddlewareState<Repo, Search, Terminology>>> {
        for executor in self.0.iter() {
            if executor.code() == code {
                return Some(executor.as_ref());
            }
        }
        None
    }
}

pub struct Middleware<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
> {
    operations: ServerOperations<ServerMiddlewareState<Repo, Search, Terminology>>,
}

impl<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
> Middleware<Repo, Search, Terminology>
{
    pub fn new() -> Self {
        Middleware {
            operations: ServerOperations::new(),
        }
    }
}

fn get_request_operation_code<'a>(request: &'a FHIRRequest) -> Option<&'a str> {
    match request {
        FHIRRequest::InvokeInstance(instance_request) => Some(&instance_request.operation.name()),
        FHIRRequest::InvokeType(type_request) => Some(&type_request.operation.name()),
        FHIRRequest::InvokeSystem(system_request) => Some(&system_request.operation.name()),
        _ => None,
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
    > for Middleware<Repo, Search, Terminology>
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
            if let Some(code) = get_request_operation_code(&context.request)
                && let Some(op_executor) = executors.find_operation(code)
            {
                let output: Resource = match &context.request {
                    FHIRRequest::InvokeInstance(instance_request) => {
                        let output = op_executor
                            .execute(state, instance_request.parameters.clone())
                            .await?;
                        Ok(Resource::from(output))
                    }
                    FHIRRequest::InvokeType(type_request) => {
                        let output = op_executor
                            .execute(state, type_request.parameters.clone())
                            .await?;
                        Ok(Resource::from(output))
                    }
                    FHIRRequest::InvokeSystem(system_request) => {
                        let output = op_executor
                            .execute(state, system_request.parameters.clone())
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
            } else {
                Err(OperationOutcomeError::fatal(
                    IssueType::NotFound(None),
                    "Operation not found".to_string(),
                ))
            }
        })
    }
}
