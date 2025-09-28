use std::pin::Pin;

use oxidized_fhir_model::r4::generated::resources::{Parameters, ParametersParameter};
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
    I: TryFrom<Vec<ParametersParameter>, Error = OperationOutcomeError>
        + Into<Vec<ParametersParameter>>,
    O: TryFrom<Vec<ParametersParameter>, Error = OperationOutcomeError>
        + Into<Vec<ParametersParameter>>,
>
{
    fn execute(
        &self,
        input: Param<I>,
    ) -> impl Future<Output = Result<O, OperationOutcomeError>> + Send + Sync;
}

pub struct OperationExecutor<
    I: TryFrom<Vec<ParametersParameter>, Error = OperationOutcomeError>
        + Into<Vec<ParametersParameter>>
        + Send
        + Sync,
    O: TryFrom<Vec<ParametersParameter>, Error = OperationOutcomeError>
        + Into<Vec<ParametersParameter>>
        + Send
        + Sync,
> {
    executor: Box<
        dyn Fn(I) -> Pin<Box<dyn Future<Output = Result<O, OperationOutcomeError>> + Send + Sync>>
            + Send
            + Sync,
    >,
}

impl<
    I: TryFrom<Vec<ParametersParameter>, Error = OperationOutcomeError>
        + Into<Vec<ParametersParameter>>
        + Send
        + Sync,
    O: TryFrom<Vec<ParametersParameter>, Error = OperationOutcomeError>
        + Into<Vec<ParametersParameter>>
        + Send
        + Sync,
> OperationExecutor<I, O>
{
    pub fn new(
        executor: Box<
            dyn Fn(
                    I,
                )
                    -> Pin<Box<dyn Future<Output = Result<O, OperationOutcomeError>> + Send + Sync>>
                + Send
                + Sync,
        >,
    ) -> Self {
        Self { executor }
    }
}

impl<
    I: TryFrom<Vec<ParametersParameter>, Error = OperationOutcomeError>
        + Into<Vec<ParametersParameter>>
        + Send
        + Sync,
    O: TryFrom<Vec<ParametersParameter>, Error = OperationOutcomeError>
        + Into<Vec<ParametersParameter>>
        + Send
        + Sync,
> OperationInvocation<I, O> for OperationExecutor<I, O>
{
    async fn execute(&self, input: Param<I>) -> Result<O, OperationOutcomeError> {
        let input = match input {
            Param::Parameters(params) => I::try_from(params.parameter.unwrap_or_default()),
            Param::Value(v) => Ok(v),
        }?;

        let output = (self.executor)(input).await;
        output
    }
}
