use haste_fhir_client::canonical_resolver::CanonicalResolver;
use haste_fhir_model::r4::generated::{
    resources::{RUST_TO_FHIR_TYPE_MAP, Resource, ResourceType, StructureDefinition},
    terminology::IssueType,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_reflect::MetaValue;
use std::sync::Arc;

pub struct FHIRProfilerCTX<Resolver: CanonicalResolver> {
    resolver: Arc<Resolver>,
}
impl<Resolver: CanonicalResolver> FHIRProfilerCTX<Resolver> {
    pub fn new(resolver: Arc<Resolver>) -> Self {
        Self { resolver }
    }
}

#[allow(dead_code)]
/// Helper function to get the FHIR type from a MetaValue
/// Internally on types we generate hashmap of rust type name to FHIR type, so we can use that to get the FHIR type for a given MetaValue.
fn get_fhir_type(value: &dyn MetaValue) -> Option<&'static str> {
    RUST_TO_FHIR_TYPE_MAP.get(value.typename()).map(|s| *s)
}

pub async fn validate_profile(
    _profile_ctx: FHIRProfilerCTX<impl CanonicalResolver>,
    _sd: &StructureDefinition,
    _values: Vec<&dyn MetaValue>,
) -> Result<(), OperationOutcomeError> {
    Ok(())
}

pub async fn validate_profile_by_url<Resolver: CanonicalResolver>(
    profile_ctx: FHIRProfilerCTX<Resolver>,
    canonical_url: &str,
    values: Vec<&dyn MetaValue>,
) -> Result<(), OperationOutcomeError> {
    let Some(profile) = profile_ctx
        .resolver
        .resolve(ResourceType::StructureDefinition, canonical_url)
        .await?
    else {
        return Err(OperationOutcomeError::error(
            IssueType::NotFound(None),
            format!("Profile with url '{}' not found", canonical_url),
        ));
    };

    match &*profile {
        Resource::StructureDefinition(sd) => validate_profile(profile_ctx, sd, values).await,
        _ => Err(OperationOutcomeError::error(
            IssueType::Invalid(None),
            format!(
                "Resource at url '{}' is not a StructureDefinition",
                canonical_url
            ),
        )),
    }
}
