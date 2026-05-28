use haste_fhir_operation_error::OperationOutcomeError;

pub trait Worker {
    fn run(&self) -> impl Future<Output = Result<(), OperationOutcomeError>>;
}
