use haste_fhir_operation_error::OperationOutcomeError;
use haste_indexing_worker::run_worker;

pub async fn worker() -> Result<(), OperationOutcomeError> {
    run_worker().await
}
