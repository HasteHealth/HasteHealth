#![allow(dead_code)]

use std::sync::Arc;

use haste_fhir_model::r4::generated::{
    resources::Resource,
    terminology::{AllTypes, CanonicalResourceTypes},
};

trait CanonicalResolver {
    fn resolve(
        &self,
        canonical_type: &CanonicalResourceTypes,
        url: &str,
    ) -> dyn Future<Output = Option<Arc<Resource>>>;
}

struct FHIRProfilerCTX {
    resolver: Arc<dyn CanonicalResolver>,
}

pub async fn validate_profile(_fhir_type: &AllTypes, _url: &str) -> String {
    "Hello, FHIR Profiling!".to_string()
}
