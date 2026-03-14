use haste_fhir_client::canonical_resolver::CanonicalResolver;
use haste_fhir_model::r4::generated::{
    resources::{OperationOutcome, Resource, ResourceType, StructureDefinition},
    terminology::{IssueType, TypeDerivationRule},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_pointer::Path;
use haste_reflect::MetaValue;
use std::sync::Arc;

use crate::element::validate_element;

mod element;
mod slicing;
mod utilities;
mod validators;

pub struct FHIRProfileArguments<Resolver: CanonicalResolver> {
    resolver: Arc<Resolver>,
}

impl<Resolver: CanonicalResolver> FHIRProfileArguments<Resolver> {
    pub fn new(resolver: Arc<Resolver>) -> Self {
        Self { resolver }
    }
}

pub struct FHIRProfileCTX<'a, Resolver: CanonicalResolver> {
    resolver: Arc<Resolver>,
    profile: &'a StructureDefinition,
    root: &'a dyn MetaValue,
}

impl<'a, Resolver: CanonicalResolver> FHIRProfileCTX<'a, Resolver> {
    pub fn new(
        resolver: Arc<Resolver>,
        profile: &'a StructureDefinition,
        root: &'a dyn MetaValue,
    ) -> Self {
        Self {
            resolver,
            profile,
            root,
        }
    }
}

pub async fn validate_profile<'a>(
    ctx: Arc<FHIRProfileCTX<'a, impl CanonicalResolver>>,
) -> Result<OperationOutcome, OperationOutcomeError> {
    let mut outcome = OperationOutcome::default();
    match ctx.profile.derivation.as_ref().map(|d| d.as_ref()) {
        Some(TypeDerivationRule::Constraint(_)) => {
            let element_location = Path::new()
                .descend("snapshot")
                .descend("element")
                .descend("0");

            let starting_path = Path::new();

            let result = validate_element(ctx, &element_location, &starting_path).await?;
            outcome.issue.extend(result);
        }
        _ => {
            return Err(OperationOutcomeError::error(
                IssueType::Invalid(None),
                "Only profiles with derivation 'constraint' are supported".to_string(),
            ));
        }
    }

    Ok(outcome)
}

pub async fn validate_profile_by_url<'a>(
    args: FHIRProfileArguments<impl CanonicalResolver>,
    canonical_url: &str,
    value: &'a dyn MetaValue,
) -> Result<(), OperationOutcomeError> {
    let Some(profile) = args
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
        Resource::StructureDefinition(sd) => {
            validate_profile(Arc::new(FHIRProfileCTX {
                resolver: args.resolver.clone(),
                profile: sd,
                root: value,
            }))
            .await?;

            Ok(())
        }
        _ => Err(OperationOutcomeError::error(
            IssueType::Invalid(None),
            format!(
                "Resource at url '{}' is not a StructureDefinition",
                canonical_url
            ),
        )),
    }
}
