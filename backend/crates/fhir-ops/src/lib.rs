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
    ) -> impl Future<Output = Result<Param<O>, OperationOutcomeError>> + Send + Sync;
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
        dyn Fn(
                Param<I>,
            ) -> Pin<
                Box<dyn Future<Output = Result<Param<O>, OperationOutcomeError>> + Send + Sync>,
            > + Send
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
                    Param<I>,
                ) -> Pin<
                    Box<dyn Future<Output = Result<Param<O>, OperationOutcomeError>> + Send + Sync>,
                > + Send
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
    async fn execute(&self, input: Param<I>) -> Result<Param<O>, OperationOutcomeError> {
        let output = (self.executor)(input).await;
        output
    }
}
