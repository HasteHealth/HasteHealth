use std::sync::Arc;

use haste_fhir_model::r4::generated::resources::{Resource, ResourceType};

pub mod remote;
pub trait CanonicalResolver<CTX, Error> {
    fn resolve(
        &self,
        ctx: CTX,
        resource_type: ResourceType,
        canonical_url: String,
    ) -> impl Future<Output = Result<Option<Arc<Resource>>, Error>>;
}
