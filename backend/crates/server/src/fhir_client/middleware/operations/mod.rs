use crate::fhir_client::{
    ServerCTX,
    middleware::{
        ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput,
        ServerMiddlewareState,
    },
};
use haste_fhir_client::{
    FHIRClient,
    middleware::MiddlewareChain,
    request::{
        FHIRInvokeSystemResponse, FHIRRequest, FHIRResponse, InvocationRequest, InvokeResponse,
    },
};
use haste_fhir_model::r4::generated::{
    resources::{Parameters, Resource},
    terminology::IssueType,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_ops::OperationInvocation;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::Repository;
use std::sync::Arc;

mod custom_operations;

struct ServerOperations<CTX>(Arc<Vec<Box<dyn OperationInvocation<CTX>>>>);

pub struct ServerOperationContext<
    State,
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
> {
    pub ctx: Arc<ServerCTX<Client>>,
    pub state: State,
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
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>
    ServerOperations<
        ServerOperationContext<ServerMiddlewareState<Repo, Search, Terminology>, Client>,
    >
{
    pub fn new() -> Self {
        let executors: Vec<
            Box<
                dyn OperationInvocation<
                    ServerOperationContext<
                        ServerMiddlewareState<Repo, Search, Terminology>,
                        Client,
                    >,
                >,
            >,
        > = vec![
            Box::new(custom_operations::valueset_expand_op()),
            Box::new(custom_operations::project_information_op()),
            Box::new(custom_operations::active_refresh_tokens_op()),
            Box::new(custom_operations::approved_scopes_op()),
            Box::new(custom_operations::delete_approved_scope_op()),
            Box::new(custom_operations::delete_refresh_token_op()),
            Box::new(custom_operations::endpoint_metadata_op()),
            Box::new(custom_operations::idp_registration_info_op()),
            Box::new(custom_operations::evaluate_policy_op()),
            Box::new(custom_operations::valueset_validate_code_op()),
        ];

        Self(Arc::new(executors))
    }

    pub fn find_operation(
        &self,
        request: &FHIRRequest,
    ) -> Option<
        &dyn OperationInvocation<
            ServerOperationContext<ServerMiddlewareState<Repo, Search, Terminology>, Client>,
        >,
    > {
        let code = get_request_operation_code(&request)?;
        for executor in self.0.iter() {
            if executor.code() == code {
                return Some(executor.as_ref());
            }
        }
        None
    }
}

fn get_request_operation_code<'a>(request: &'a FHIRRequest) -> Option<&'a str> {
    match request {
        FHIRRequest::Invocation(InvocationRequest::Instance(instance_request)) => {
            Some(&instance_request.operation.name())
        }
        FHIRRequest::Invocation(InvocationRequest::Type(type_request)) => {
            Some(&type_request.operation.name())
        }
        FHIRRequest::Invocation(InvocationRequest::System(system_request)) => {
            Some(&system_request.operation.name())
        }
        _ => None,
    }
}

pub struct Middleware<
    State,
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
> {
    operations: ServerOperations<ServerOperationContext<State, Client>>,
}

impl<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
> Middleware<ServerMiddlewareState<Repo, Search, Terminology>, Client>
{
    pub fn new() -> Self {
        Middleware {
            operations: ServerOperations::new(),
        }
    }
}

impl<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>
    MiddlewareChain<
        ServerMiddlewareState<Repo, Search, Terminology>,
        Arc<ServerCTX<Client>>,
        FHIRRequest,
        FHIRResponse,
        OperationOutcomeError,
    > for Middleware<ServerMiddlewareState<Repo, Search, Terminology>, Client>
{
    /// Sets tenant to search in system for artifact resources IE SDs etc..
    fn call(
        &self,
        state: ServerMiddlewareState<Repo, Search, Terminology>,
        mut context: ServerMiddlewareContext<Client>,
        _next: Option<
            Arc<ServerMiddlewareNext<Client, ServerMiddlewareState<Repo, Search, Terminology>>>,
        >,
    ) -> ServerMiddlewareOutput<Client> {
        let executors = self.operations.clone();

        // tokio::task::spawn_blocking(f)

        Box::pin(async move {
            if let Some(op_executor) = executors.find_operation(&context.request) {
                let output: Resource = match &context.request {
                    FHIRRequest::Invocation(request) => {
                        let output = op_executor
                            .execute(
                                ServerOperationContext {
                                    state,
                                    ctx: context.ctx.clone(),
                                },
                                context.ctx.tenant.clone(),
                                context.ctx.project.clone(),
                                &request,
                            )
                            .await?;
                        Ok(Resource::from(output))
                    }
                    _ => Err(OperationOutcomeError::fatal(
                        IssueType::Exception(None),
                        "Operation not supported".to_string(),
                    )),
                }?;

                context.response = Some(FHIRResponse::Invoke(InvokeResponse::System(
                    FHIRInvokeSystemResponse { resource: output },
                )));

                Ok(context)
            } else {
                match get_request_operation_code(&context.request) {
                    Some("execute-ts") => {
                        let context_clone = context.ctx.clone();

                        let (tx, rx) = tokio::sync::oneshot::channel();

                        std::thread::spawn(move || {
                            let rt = tokio::runtime::Builder::new_current_thread()
                                .enable_all()
                                .build()
                                .unwrap();

                            let local = tokio::task::LocalSet::new();

                            local.block_on(&rt, async move {
                                let output = tokio::task::spawn_local(async move {
                                    let result = haste_deno_executor::run_code(
                                        context_clone.clone(),
                                        context_clone.client.clone(),
                                        haste_deno_executor::PluginCodeType::TypeScript,
                                        r#"
                                export default async function() {
                                    const sd = await fhir.readResource("StructureDefinition", "Patient");

                                    return {
                                        resourceType: 'Parameters',
                                        parameter: [

                                            {
                                                name: 'message',
                                                resource: sd
                                            }
                                        ]
                                    };
                                }
                            "#
                                        .to_string(),
                                    )
                                    .await
                                    .map_err(|e| {
                                        tracing::error!("Error executing dynamic code: {:?}", e);
                                        OperationOutcomeError::fatal(
                                            IssueType::Exception(None),
                                            format!("Failed to execute dynamic code"),
                                        )
                                    })?;

                                    
      
                                    return result.ok_or_else(|| OperationOutcomeError::error(IssueType::Exception(None), "Dynamic code did not return a result".to_string()));
                                })
                                .await
                                .map_err(|e| {
                                    tracing::error!("Error executing dynamic code: {:?}", e);
                                    OperationOutcomeError::fatal(
                                        IssueType::Exception(None),
                                        format!("Failed to execute dynamic code"),
                                    )
                                });

                                let _ = tx.send(output);
                            });
                        });

                        // Back in the Send future — just await the result
                        let result = rx.await.map_err(|_| {
                            OperationOutcomeError::fatal(
                                IssueType::Exception(None),
                                format!("Failed to receive dynamic code execution result"),
                            )
                        })???;

                        let parameters =
                            haste_fhir_serialization_json::from_serde_value::<Parameters>(result)
                                .map_err(|_| {
                                OperationOutcomeError::fatal(
                                    IssueType::Exception(None),
                                    "Failed to deserialize dynamic code result".to_string(),
                                )
                            })?;

                        context.response = Some(FHIRResponse::Invoke(InvokeResponse::System(
                            FHIRInvokeSystemResponse {
                                resource: Resource::Parameters(parameters),
                            },
                        )));

                        Ok(context)
                    }
                    _ => Err(OperationOutcomeError::fatal(
                        IssueType::NotFound(None),
                        "Operation not found".to_string(),
                    )),
                }
            }
        })
    }
}
