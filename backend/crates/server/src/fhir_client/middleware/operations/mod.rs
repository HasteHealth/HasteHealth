use crate::fhir_client::{
    ClientState, ServerCTX,
    middleware::{Backend, Handler, Output, operation_definitions},
};
use haste_fhir_client::request::{
    FHIRInvokeInstanceResponse, FHIRInvokeSystemResponse, FHIRInvokeTypeResponse, FHIRRequest,
    FHIRResponse, InvocationRequest, InvokeResponse,
};
use haste_fhir_model::r4::generated::{
    resources::{OperationDefinition, Resource, ResourceType},
    terminology::IssueType,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_ops::{OperationInvocation, defines};
use haste_operation_executor::{
    providers::deno_embedded::pool::DenoPool, traits::OperationExecutor,
};
use std::sync::Arc;

mod custom_operations;

/// What a built-in operation runs with: the caller's context and the state
/// of the pipeline that invoked it.
pub struct ServerOperationContext<B: Backend> {
    pub ctx: Arc<ServerCTX<B>>,
    pub state: ClientState<B>,
}

type BuiltinOperation<B> = Box<dyn OperationInvocation<ServerOperationContext<B>>>;

/// Ends the operation lane: runs the server's built-in operations, and a
/// project's own OperationDefinitions through the Deno executor.
pub struct Operations<B: Backend> {
    builtins: Vec<BuiltinOperation<B>>,
    deno_pool: Arc<DenoPool>,
}

impl<B: Backend> Operations<B> {
    /// `deno_pool` should be a pool shared across all routes/clients derived
    /// from the same server rather than a freshly-constructed one -- see
    /// [`crate::fhir_client::ServerClientConfig::new`].
    pub fn new(deno_pool: Arc<DenoPool>) -> Self {
        let builtins: Vec<BuiltinOperation<B>> = vec![
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

        Operations {
            builtins,
            deno_pool,
        }
    }

    /// The OperationDefinitions of the operations implemented here, which
    /// take precedence over a project's own.
    pub fn operation_definitions(&self) -> operation_definitions::Builtins {
        self.builtins
            .iter()
            .map(|operation| operation.operation_definition())
            .collect()
    }

    /// The built-in operation whose definition defines `request`: its code,
    /// at that level, on that resource type.
    fn find_operation(
        &self,
        request: &InvocationRequest,
    ) -> Option<&dyn OperationInvocation<ServerOperationContext<B>>> {
        self.builtins
            .iter()
            .find(|operation| defines(operation.operation_definition(), request))
            .map(|operation| operation.as_ref())
    }
}

/// The project's (custom, tenant-authored) `OperationDefinition` that defines
/// `request`, if there is one. The search lists built-ins too, but none
/// defines the request or the caller would have run it instead.
async fn find_custom_operation<B: Backend>(
    ctx: &Arc<ServerCTX<B>>,
    request: &InvocationRequest,
) -> Result<Option<OperationDefinition>, OperationOutcomeError> {
    let (code, level, _) = invocation(request);
    let bundle = ctx
        .client
        .search_type(
            ctx.clone(),
            ResourceType::OperationDefinition,
            vec![
                ("code".to_string(), vec![code.to_string()]),
                (level.to_string(), vec!["true".to_string()]),
            ]
            .into(),
        )
        .await?;

    Ok(bundle
        .entry
        .as_deref()
        .unwrap_or_default()
        .iter()
        .filter_map(|entry| match entry.resource.as_deref() {
            Some(Resource::OperationDefinition(definition)) => Some(definition),
            _ => None,
        })
        .find(|definition| defines(definition, request))
        .cloned())
}

/// The error for an invocation that neither a built-in nor the project defines.
fn not_defined(request: &InvocationRequest) -> OperationOutcomeError {
    let (code, _, invoked) = invocation(request);
    OperationOutcomeError::error(
        IssueType::not_found(),
        format!("No operation '{code}' is defined {invoked}."),
    )
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

/// The response to `request`, at the level it was invoked at. The typed client
/// methods (`invoke_instance`, `invoke_type`) only accept their own level.
fn invoke_response(request: &InvocationRequest, resource: Resource) -> FHIRResponse {
    FHIRResponse::Invoke(match request {
        InvocationRequest::Instance(_) => {
            InvokeResponse::Instance(FHIRInvokeInstanceResponse { resource })
        }
        InvocationRequest::Type(_) => InvokeResponse::Type(FHIRInvokeTypeResponse { resource }),
        InvocationRequest::System(_) => {
            InvokeResponse::System(FHIRInvokeSystemResponse { resource })
        }
    })
}

impl<B: Backend> Handler<B> for Operations<B> {
    fn handle<'a>(
        &'a self,
        state: &'a ClientState<B>,
        ctx: Arc<ServerCTX<B>>,
        request: FHIRRequest,
    ) -> Output<'a> {
        Box::pin(async move {
            let FHIRRequest::Invocation(invoke_request) = &request else {
                return Err(OperationOutcomeError::fatal(
                    IssueType::exception(),
                    "Operation not supported".to_string(),
                ));
            };

            let builtin = self.find_operation(invoke_request);
            let custom = match builtin {
                Some(_) => None,
                None => Some(
                    find_custom_operation(&ctx, invoke_request)
                        .await?
                        .ok_or_else(|| not_defined(invoke_request))?,
                ),
            };

            // `Prefer: respond-async`: the operation exists and the request
            // has passed every check before this point, so queue it and
            // answer 202. The worker runs it through this same pipeline later.
            if ctx.prefer_async {
                return crate::jobs::operation::submit(
                    state.repo.as_ref(),
                    state.config.as_ref(),
                    ctx.as_ref(),
                    invoke_request,
                )
                .await;
            }

            if let Some(operation) = builtin {
                let output = operation
                    .execute(
                        ServerOperationContext {
                            state: state.clone(),
                            ctx: ctx.clone(),
                        },
                        ctx.tenant.clone(),
                        ctx.project.clone(),
                        invoke_request,
                    )
                    .await?;

                return Ok(invoke_response(invoke_request, output));
            }

            let operation_definition = custom.ok_or_else(|| not_defined(invoke_request))?;
            let output = self
                .deno_pool
                .execute_operation(
                    ctx.clone(),
                    ctx.client.clone(),
                    &operation_definition,
                    invoke_request,
                )
                .await?;

            Ok(invoke_response(
                invoke_request,
                Resource::Parameters(output),
            ))
        })
    }
}
