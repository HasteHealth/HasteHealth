use crate::fhir_client::{
    ServerCTX,
    middleware::{
        ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput,
        ServerMiddlewareState, operation_definitions,
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
    resources::{Resource, ResourceType},
    terminology::IssueType,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_ops::{OperationInvocation, defines};
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_operation_executor::{
    providers::deno_embedded::pool::DenoPool, traits::OperationExecutor,
};
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
            Box::new(custom_operations::view_definition_run()),
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
            Box::new(custom_operations::hl7v2_parse_op()),
            Box::new(custom_operations::indexing_errors_op()),
            Box::new(custom_operations::tenant_customization_op()),
            Box::new(custom_operations::tenant_branding_op()),
            Box::new(custom_operations::patient_everything()),
            Box::new(custom_operations::smart_launch_op()),
        ];

        Self(Arc::new(executors))
    }

    /// The built-in operation whose definition defines `request`: its code,
    /// at that level, on that resource type.
    pub fn find_operation(
        &self,
        request: &InvocationRequest,
    ) -> Option<
        &dyn OperationInvocation<
            ServerOperationContext<ServerMiddlewareState<Repo, Search, Terminology>, Client>,
        >,
    > {
        self.0
            .iter()
            .find(|operation| defines(operation.operation_definition(), request))
            .map(|operation| operation.as_ref())
    }
}

/// What `request` invokes: the code, the OperationDefinition search parameter
/// for its level, and where, for an error message.
fn invocation(request: &InvocationRequest) -> (&str, &'static str, String) {
    match request {
        InvocationRequest::Instance(request) => (
            request.operation.name(),
            "instance",
            format!("on {} instances", request.resource_type.as_ref()),
        ),
        InvocationRequest::Type(request) => (
            request.operation.name(),
            "type",
            format!("on the {} type", request.resource_type.as_ref()),
        ),
        InvocationRequest::System(request) => (
            request.operation.name(),
            "system",
            "at the system level".to_string(),
        ),
    }
}

pub struct Middleware<
    State,
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
> {
    operations: ServerOperations<ServerOperationContext<State, Client>>,
    deno_executor: Arc<DenoPool>,
}

impl<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
> Middleware<ServerMiddlewareState<Repo, Search, Terminology>, Client>
{
    /// `deno_executor` should be a pool shared across all routes/clients derived
    /// from the same server rather than a freshly-constructed one -- see
    /// [`crate::fhir_client::ServerClientConfig::new`].
    pub fn new(deno_executor: Arc<DenoPool>) -> Self {
        Middleware {
            operations: ServerOperations::new(),
            deno_executor,
        }
    }

    /// The OperationDefinitions of the operations implemented here, which
    /// take precedence over a project's own.
    pub fn operation_definitions(&self) -> operation_definitions::Builtins {
        self.operations
            .0
            .iter()
            .map(|operation| operation.operation_definition())
            .collect()
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
        let deno_executor = self.deno_executor.clone();

        // tokio::task::spawn_blocking(f)

        Box::pin(async move {
            match &context.request {
                FHIRRequest::Invocation(invoke_request) => {
                    if let Some(op_executor) = executors.find_operation(invoke_request) {
                        let output = op_executor
                            .execute(
                                ServerOperationContext {
                                    state,
                                    ctx: context.ctx.clone(),
                                },
                                context.ctx.tenant.clone(),
                                context.ctx.project.clone(),
                                invoke_request,
                            )
                            .await?;

                        context.response = Some(FHIRResponse::Invoke(InvokeResponse::System(
                            FHIRInvokeSystemResponse { resource: output },
                        )));

                        Ok(context)
                    } else {
                        // The project's definitions of this code at this
                        // level. The search lists built-ins too, but none
                        // defines this request or it would have run above.
                        let (code, level, invoked) = invocation(invoke_request);
                        let bundle = context
                            .ctx
                            .client
                            .search_type(
                                context.ctx.clone(),
                                ResourceType::OperationDefinition,
                                vec![
                                    ("code".to_string(), vec![code.to_string()]),
                                    (level.to_string(), vec!["true".to_string()]),
                                ]
                                .into(),
                            )
                            .await?;

                        let Some(operation_definition) = bundle
                            .entry
                            .as_deref()
                            .unwrap_or_default()
                            .iter()
                            .filter_map(|entry| match entry.resource.as_deref() {
                                Some(Resource::OperationDefinition(definition)) => Some(definition),
                                _ => None,
                            })
                            .find(|definition| defines(definition, invoke_request))
                        else {
                            return Err(OperationOutcomeError::error(
                                IssueType::not_found(),
                                format!("No operation '{code}' is defined {invoked}."),
                            ));
                        };

                        let output = deno_executor
                            .execute_operation(
                                context.ctx.clone(),
                                context.ctx.client.clone(),
                                operation_definition,
                                invoke_request,
                            )
                            .await?;

                        context.response = Some(FHIRResponse::Invoke(InvokeResponse::System(
                            FHIRInvokeSystemResponse {
                                resource: Resource::Parameters(output),
                            },
                        )));

                        Ok(context)
                    }
                }
                _ => Err(OperationOutcomeError::fatal(
                    IssueType::exception(),
                    "Operation not supported".to_string(),
                )),
            }
        })
    }
}
