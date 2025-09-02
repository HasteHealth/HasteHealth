use oxidized_fhir_model::r4::types::Parameters;
use oxidized_fhir_operation_error::OperationOutcomeError;

pub enum Param<I: TryFrom<Parameters, Error = OperationOutcomeError>> {
    Value(I),
    Parameters(Parameters),
}

pub trait OperationInvocation<
    I: TryFrom<Parameters, Error = OperationOutcomeError>,
    Output: TryFrom<Parameters, Error = OperationOutcomeError>,
>
{
    fn new(
        executor: Box<
            dyn Fn(Param<I>) -> Result<Param<Output>, OperationOutcomeError> + Send + Sync,
        >,
    ) -> Self;

    fn execute(&self, input: Param<I>) -> Result<Param<Output>, OperationOutcomeError>;
}
