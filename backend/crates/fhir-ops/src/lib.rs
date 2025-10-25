use std::{pin::Pin, sync::Arc};

use oxidized_fhir_model::r4::generated::resources::{Parameters, ParametersParameter, Resource};
use oxidized_fhir_operation_error::OperationOutcomeError;

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

pub trait OperationInvocation<
    CTX: Send,
    I: TryFrom<Vec<ParametersParameter>, Error = OperationOutcomeError>
        + Into<Vec<ParametersParameter>>
        + Send,
    O: TryFrom<Vec<ParametersParameter>, Error = OperationOutcomeError> + Into<Resource> + Send,
>
{
    fn execute(
        &self,
        ctx: CTX,
        input: Param<I>,
    ) -> impl Future<Output = Result<O, OperationOutcomeError>> + Send;
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
            dyn Fn(CTX, I) -> Pin<Box<dyn Future<Output = Result<O, OperationOutcomeError>> + Send>>
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
            dyn Fn(CTX, I) -> Pin<Box<dyn Future<Output = Result<O, OperationOutcomeError>> + Send>>
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
    CTX: Send,
    I: TryFrom<Vec<ParametersParameter>, Error = OperationOutcomeError>
        + Into<Vec<ParametersParameter>>
        + Send,
    O: TryFrom<Vec<ParametersParameter>, Error = OperationOutcomeError> + Into<Resource> + Send,
> OperationInvocation<CTX, I, O> for OperationExecutor<CTX, I, O>
{
    fn execute(
        &self,
        ctx: CTX,
        input: Param<I>,
    ) -> impl Future<Output = Result<O, OperationOutcomeError>> + Send {
        let executor = self.executor.clone();

        async move {
            let input = match input {
                Param::Parameters(params) => I::try_from(params.parameter.unwrap_or_default()),
                Param::Value(v) => Ok(v),
            }?;

            let output = (executor)(ctx, input).await;
            output
        }
    }

    fn code<'a>(&'a self) -> &'a str {
        &self.code
    }
}
