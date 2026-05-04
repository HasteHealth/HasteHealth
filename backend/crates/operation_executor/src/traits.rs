use haste_fhir_client::{FHIRClient, request::InvocationRequest};
use haste_fhir_model::r4::generated::resources::{OperationDefinition, Parameters};
use haste_fhir_operation_error::OperationOutcomeError;
use std::sync::Arc;

pub trait OperationExecutor {
    fn execute_operation<
        CTX: Clone + Send + 'static,
        Client: FHIRClient<CTX, OperationOutcomeError> + 'static,
    >(
        &self,
        context: CTX,
        client: Arc<Client>,
        operation: &OperationDefinition,
        input: &InvocationRequest,
    ) -> impl Future<Output = Result<Parameters, OperationOutcomeError>>;
}
