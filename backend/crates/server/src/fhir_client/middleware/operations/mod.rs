use crate::fhir_client::{
    ServerCTX,
    middleware::{
        ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput,
        ServerMiddlewareState,
    },
};
use haste_fhir_client::{
    middleware::MiddlewareChain,
    request::{FHIRInvokeSystemResponse, FHIRRequest, FHIRResponse},
};
use haste_fhir_model::r4::generated::{resources::Resource, terminology::IssueType};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_ops::OperationInvocation;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::Repository;
use std::sync::Arc;

mod custom_operations;

struct ServerOperations<CTX>(Arc<Vec<Box<dyn OperationInvocation<CTX>>>>);

pub struct ServerOperationContext<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
> {
    pub ctx: Arc<ServerCTX<Repo, Search, Terminology>>,
    pub state: ServerMiddlewareState<Repo, Search, Terminology>,
}

impl<CTX> Clone for ServerOperations<CTX> {
    fn clone(&self) -> Self {
        ServerOperations(self.0.clone())
    }
}

impl<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
> ServerOperations<ServerOperationContext<Repo, Search, Terminology>>
{
    pub fn new() -> Self {
        let executors: Vec<
            Box<dyn OperationInvocation<ServerOperationContext<Repo, Search, Terminology>>>,
        > = vec![
            Box::new(custom_operations::valueset_expand()),
            Box::new(custom_operations::project_information()),
            Box::new(custom_operations::active_refresh_tokens()),
            Box::new(custom_operations::approved_scopes()),
            Box::new(custom_operations::delete_approved_scope()),
            Box::new(custom_operations::delete_refresh_token()),
        ];

        Self(Arc::new(executors))
    }

    pub fn find_operation(
        &self,
        code: &str,
    ) -> Option<&dyn OperationInvocation<ServerOperationContext<Repo, Search, Terminology>>> {
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
    operations: ServerOperations<ServerOperationContext<Repo, Search, Terminology>>,
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
        Arc<ServerCTX<Repo, Search, Terminology>>,
        FHIRRequest,
        FHIRResponse,
        OperationOutcomeError,
    > for Middleware<Repo, Search, Terminology>
{
    /// Sets tenant to search in system for artifact resources IE SDs etc..
    fn call(
        &self,
        state: ServerMiddlewareState<Repo, Search, Terminology>,
        mut context: ServerMiddlewareContext<Repo, Search, Terminology>,
        _next: Option<Arc<ServerMiddlewareNext<Repo, Search, Terminology>>>,
    ) -> ServerMiddlewareOutput<Repo, Search, Terminology> {
        let executors = self.operations.clone();
        Box::pin(async move {
            if let Some(code) = get_request_operation_code(&context.request)
                && let Some(op_executor) = executors.find_operation(code)
            {
                let output: Resource = match &context.request {
                    FHIRRequest::InvokeInstance(instance_request) => {
                        let output = op_executor
                            .execute(
                                ServerOperationContext {
                                    state,
                                    ctx: context.ctx.clone(),
                                },
                                context.ctx.tenant.clone(),
                                context.ctx.project.clone(),
                                instance_request.parameters.clone(),
                            )
                            .await?;
                        Ok(Resource::from(output))
                    }
                    FHIRRequest::InvokeType(type_request) => {
                        let output = op_executor
                            .execute(
                                ServerOperationContext {
                                    state,
                                    ctx: context.ctx.clone(),
                                },
                                context.ctx.tenant.clone(),
                                context.ctx.project.clone(),
                                type_request.parameters.clone(),
                            )
                            .await?;
                        Ok(Resource::from(output))
                    }
                    FHIRRequest::InvokeSystem(system_request) => {
                        let output = op_executor
                            .execute(
                                ServerOperationContext {
                                    state,
                                    ctx: context.ctx.clone(),
                                },
                                context.ctx.tenant.clone(),
                                context.ctx.project.clone(),
                                system_request.parameters.clone(),
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
            } else {
                Err(OperationOutcomeError::fatal(
                    IssueType::NotFound(None),
                    "Operation not found".to_string(),
                ))
            }
        })
    }
}
