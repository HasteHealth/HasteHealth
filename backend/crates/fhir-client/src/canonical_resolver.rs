use std::sync::Arc;

use haste_fhir_model::r4::generated::resources::{Resource, ResourceType};
use haste_fhir_operation_error::OperationOutcomeError;

pub trait CanonicalResolver {
    fn resolve(
        &self,
        resource_type: ResourceType,
        canonical_url: String,
    ) -> impl Future<Output = Result<Option<Arc<Resource>>, OperationOutcomeError>> + Send;
}
