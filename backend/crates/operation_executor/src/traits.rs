use haste_fhir_client::{
    FHIRClient,
    request::{InvocationRequest, InvokeResponse},
};
use haste_fhir_model::r4::generated::resources::OperationDefinition;
use haste_fhir_operation_error::OperationOutcomeError;
use std::sync::Arc;

pub trait OperationExecutor {
    fn execute_operation<CTX, Client: FHIRClient<CTX, OperationOutcomeError>>(
        &self,
        context: CTX,
        client: Arc<Client>,
        operation: &OperationDefinition,
        input: InvocationRequest,
    ) -> impl Future<Output = Result<InvokeResponse, OperationOutcomeError>>;
}
