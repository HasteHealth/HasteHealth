use haste_fhir_client::canonical_resolver::CanonicalResolver;
use haste_fhir_model::r4::generated::terminology::AllTypes;
use haste_fhir_operation_error::OperationOutcomeError;
use std::sync::Arc;

pub struct FHIRProfilerCTX<Resolver: CanonicalResolver> {
    #[allow(dead_code)]
    resolver: Arc<Resolver>,
}
impl<Resolver: CanonicalResolver> FHIRProfilerCTX<Resolver> {
    pub fn new(resolver: Arc<Resolver>) -> Self {
        Self { resolver }
    }
}

pub async fn validate_profile<Resolver: CanonicalResolver>(
    _profile_ctx: FHIRProfilerCTX<Resolver>,
    _fhir_type: &AllTypes,
    _url: &str,
) -> Result<(), OperationOutcomeError> {
    Ok(())
}
