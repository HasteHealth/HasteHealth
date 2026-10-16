use std::{pin::Pin, sync::Arc};

use haste_fhir_client::request::InvocationRequest;
use haste_fhir_model::r4::generated::{
    resources::{OperationDefinition, Parameters, ParametersParameter, Resource},
    terminology::{BoundCode, ResourceTypes},
    types::FHIRBoolean,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId};

#[cfg(feature = "derive")]
pub mod derive;

pub enum Param<
    T: TryFrom<Vec<ParametersParameter>, Error = OperationOutcomeError>
        + Into<Vec<ParametersParameter>>,
> {
    Value(T),
    Parameters(Parameters),
}

impl<
    T: TryFrom<Vec<ParametersParameter>, Error = OperationOutcomeError>
        + Into<Vec<ParametersParameter>>,
> Param<T>
{
    pub fn as_parameters(self) -> Parameters {
        match self {
            Param::Value(v) => Parameters {
                parameter: Some(v.into()),
                ..Default::default()
            },
            Param::Parameters(p) => p,
        }
    }
}

fn allows(level: &FHIRBoolean) -> bool {
    level.value.unwrap_or(false)
}

/// The resource types `definition` is defined on, or `None` for any.
fn resource_types(definition: &OperationDefinition) -> Option<&[BoundCode<ResourceTypes>]> {
    definition
        .resource
        .as_deref()
        .filter(|types| !types.is_empty())
}

/// Whether `definition` defines the operation `request` invokes: its code, at
/// a level it allows, on a resource type it's defined on.
#[must_use]
pub fn defines(definition: &OperationDefinition, request: &InvocationRequest) -> bool {
    let (operation, level, resource_type) = match request {
        InvocationRequest::System(request) => (&request.operation, &definition.system, None),
        InvocationRequest::Type(request) => (
            &request.operation,
            &definition.type_,
            Some(&request.resource_type),
        ),
        InvocationRequest::Instance(request) => (
            &request.operation,
            &definition.instance,
            Some(&request.resource_type),
        ),
    };

    definition.code.value.as_deref() == Some(operation.name())
        && allows(level)
        && resource_type.is_none_or(|resource_type| {
            resource_types(definition).is_none_or(|types| {
                types
                    .iter()
                    .any(|listed| listed.as_str() == Some(resource_type.as_ref()))
            })
        })
}

/// Whether `a` and `b` both define some invocation: the same code at a level
/// both allow, on a resource type both are defined on below the system level.
#[must_use]
pub fn overlaps(a: &OperationDefinition, b: &OperationDefinition) -> bool {
    let same_code = a.code.value.is_some() && a.code.value == b.code.value;
    let shared_resource_type = match (resource_types(a), resource_types(b)) {
        (Some(a), Some(b)) => a.iter().any(|resource_type| b.contains(resource_type)),
        _ => true,
    };

    same_code
        && ((allows(&a.system) && allows(&b.system))
            || (shared_resource_type
                && ((allows(&a.type_) && allows(&b.type_))
                    || (allows(&a.instance) && allows(&b.instance)))))
}

pub trait OperationInvocation<CTX: Send>: Send + Sync {
    fn execute<'a>(
        &self,
        ctx: CTX,
        tenant: TenantId,
        project: ProjectId,
        request: &'a InvocationRequest,
    ) -> Pin<Box<dyn Future<Output = Result<Resource, OperationOutcomeError>> + Send + 'a>>;
    /// The `OperationDefinition` this operation implements.
    fn operation_definition(&self) -> &'static OperationDefinition;
}

type Executor<CTX, I, O> = Box<
    dyn Fn(
            CTX,
            TenantId,
            ProjectId,
            &InvocationRequest,
            I,
        ) -> Pin<Box<dyn Future<Output = Result<O, OperationOutcomeError>> + Send>>
        + Send
        + Sync,
>;

pub struct OperationExecutor<
    CTX: Send,
    I: TryFrom<Vec<ParametersParameter>, Error = OperationOutcomeError>
        + Into<Vec<ParametersParameter>>
        + Send,
    O: TryFrom<Vec<ParametersParameter>, Error = OperationOutcomeError> + Into<Resource> + Send,
> {
    _ctx: std::marker::PhantomData<CTX>,
    operation_definition: &'static OperationDefinition,
    executor: Arc<Executor<CTX, I, O>>,
}

impl<
    CTX: Send,
    I: TryFrom<Vec<ParametersParameter>, Error = OperationOutcomeError>
        + Into<Vec<ParametersParameter>>
        + Send,
    O: TryFrom<Vec<ParametersParameter>, Error = OperationOutcomeError> + Into<Resource> + Send,
> OperationExecutor<CTX, I, O>
{
    /// Runs the operation `operation_definition` (a generated module's
    /// `OPERATION_DEFINITION`) defines, when it's invoked where the definition
    /// allows.
    #[must_use]
    pub fn new(
        operation_definition: &'static OperationDefinition,
        executor: Executor<CTX, I, O>,
    ) -> Self {
        Self {
            _ctx: std::marker::PhantomData,
            executor: Arc::new(executor),
            operation_definition,
        }
    }
}

impl<
    CTX: Send + Sync + 'static,
    I: TryFrom<Vec<ParametersParameter>, Error = OperationOutcomeError>
        + Into<Vec<ParametersParameter>>
        + Send
        + 'static,
    O: TryFrom<Vec<ParametersParameter>, Error = OperationOutcomeError>
        + Into<Resource>
        + Send
        + 'static,
> OperationInvocation<CTX> for OperationExecutor<CTX, I, O>
{
    fn execute<'a>(
        &self,
        ctx: CTX,
        tenant: TenantId,
        project: ProjectId,
        request: &'a InvocationRequest,
    ) -> Pin<Box<dyn Future<Output = Result<Resource, OperationOutcomeError>> + Send + 'a>> {
        let executor = self.executor.clone();
        Box::pin(async move {
            let parameters = match request {
                InvocationRequest::Instance(instance_request) => &instance_request.parameters,
                InvocationRequest::Type(type_request) => &type_request.parameters,
                InvocationRequest::System(system_request) => &system_request.parameters,
            };

            let input = I::try_from(parameters.parameter.clone().unwrap_or_default())?;

            let output = (executor)(ctx, tenant, project, request, input).await?;

            Ok(output.into())
        })
    }

    fn operation_definition(&self) -> &'static OperationDefinition {
        self.operation_definition
    }
}
