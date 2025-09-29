use oxidized_fhir_model::r4::generated::resources::{Resource, ResourceType};
use oxidized_fhir_operation_error::OperationOutcomeError;
use std::pin::Pin;

pub mod remote;
pub trait CanonicalResolver {
    fn resolve(
        &self,
        resource_type: ResourceType,
        id: String,
    ) -> Pin<Box<dyn Future<Output = Result<Resource, OperationOutcomeError>> + Send>>;
}
