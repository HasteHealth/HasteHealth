use std::sync::Arc;

use haste_fhir_model::r4::generated::{
    resources::{Resource, ResourceType},
    terminology::AllTypes,
};

trait CanonicalResolver {
    fn resolve(
        &self,
        fhir_type: &ResourceType,
        url: &str,
    ) -> dyn Future<Output = Option<Arc<Resource>>>;
}

struct FHIRProfilerCTX {
    resolver: Arc<dyn CanonicalResolver>,
}

pub async fn validate_profile(fhir_type: &AllTypes, url: &str) -> String {
    "Hello, FHIR Profiling!".to_string()
}
