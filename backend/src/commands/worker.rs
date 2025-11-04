use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_indexing_worker::run_worker;

pub async fn worker() -> Result<(), OperationOutcomeError> {
    run_worker().await
}
