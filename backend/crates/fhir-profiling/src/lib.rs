use haste_fhir_client::canonical_resolver::CanonicalResolver;
use haste_fhir_model::r4::generated::{
    resources::{OperationOutcome, Resource, ResourceType, StructureDefinition},
    terminology::{IssueType, TypeDerivationRule},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_pointer::Path;
use haste_reflect::MetaValue;
use std::sync::Arc;

mod element;

pub struct FHIRProfilerCTX<Resolver: CanonicalResolver> {
    resolver: Arc<Resolver>,
}
impl<Resolver: CanonicalResolver> FHIRProfilerCTX<Resolver> {
    pub fn new(resolver: Arc<Resolver>) -> Self {
        Self { resolver }
    }
}

pub async fn validate_profile(
    ctx: FHIRProfilerCTX<impl CanonicalResolver>,
    profile: &StructureDefinition,
    root: &dyn MetaValue,
) -> Result<OperationOutcome, OperationOutcomeError> {
    match profile.derivation.as_ref() {
        Some(TypeDerivationRule::Constraint(_)) => {
            let profile_location = Path::new()
                .descend("snapshot")
                .descend("element")
                .descend("0");

            let k = profile_location.ascend();
            let starting_path = Path::new();
        }
        _ => {
            return Err(OperationOutcomeError::error(
                IssueType::Invalid(None),
                "Only profiles with derivation 'constraint' are supported".to_string(),
            ));
        }
    }

    Ok(OperationOutcome::default())
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
