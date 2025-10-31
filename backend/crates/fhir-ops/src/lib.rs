use std::{pin::Pin, sync::Arc};

use oxidized_fhir_model::r4::generated::resources::{Parameters, ParametersParameter, Resource};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_jwt::{ProjectId, TenantId};

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

pub trait OperationInvocation<CTX: Send>: Send + Sync {
    fn execute(
        &self,
        ctx: CTX,
        tenant: TenantId,
        project: ProjectId,
        input: Parameters,
    ) -> Pin<Box<dyn Future<Output = Result<Resource, OperationOutcomeError>> + Send>>;
    fn code<'a>(&'a self) -> &'a str;
}

pub struct OperationExecutor<
    CTX: Send,
    I: TryFrom<Vec<ParametersParameter>, Error = OperationOutcomeError>
        + Into<Vec<ParametersParameter>>
        + Send,
    O: TryFrom<Vec<ParametersParameter>, Error = OperationOutcomeError> + Into<Resource> + Send,
> {
    _ctx: std::marker::PhantomData<CTX>,
    code: String,
    executor: Arc<
        Box<
            dyn Fn(
                    CTX,
                    TenantId,
                    ProjectId,
                    I,
                )
                    -> Pin<Box<dyn Future<Output = Result<O, OperationOutcomeError>> + Send>>
                + Send
                + Sync,
        >,
    >,
}

impl<
    CTX: Send,
    I: TryFrom<Vec<ParametersParameter>, Error = OperationOutcomeError>
        + Into<Vec<ParametersParameter>>
        + Send,
    O: TryFrom<Vec<ParametersParameter>, Error = OperationOutcomeError> + Into<Resource> + Send,
> OperationExecutor<CTX, I, O>
{
    pub fn new(
        code: String,
        executor: Box<
            dyn Fn(
                    CTX,
                    TenantId,
                    ProjectId,
                    I,
                )
                    -> Pin<Box<dyn Future<Output = Result<O, OperationOutcomeError>> + Send>>
                + Send
                + Sync,
        >,
    ) -> Self {
        Self {
            _ctx: std::marker::PhantomData,
            executor: Arc::new(executor),
            code,
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
    fn execute(
        &self,
        ctx: CTX,
        tenant: TenantId,
        project: ProjectId,
        input: Parameters,
    ) -> Pin<Box<dyn Future<Output = Result<Resource, OperationOutcomeError>> + Send>> {
        let executor = self.executor.clone();
        Box::pin(async move {
            let input = I::try_from(input.parameter.unwrap_or_default())?;

            let output = (executor)(ctx, tenant, project, input).await?;

            Ok(output.into())
        })
    }

    fn code<'a>(&'a self) -> &'a str {
        &self.code
    }
}
