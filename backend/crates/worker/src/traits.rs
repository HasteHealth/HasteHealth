use haste_fhir_operation_error::OperationOutcomeError;
use tokio::task::JoinHandle;

pub trait Worker {
    fn run(&self) -> impl Future<Output = Result<JoinHandle<()>, OperationOutcomeError>>;
    fn stop(&mut self) -> impl Future<Output = Result<(), OperationOutcomeError>>;
}
