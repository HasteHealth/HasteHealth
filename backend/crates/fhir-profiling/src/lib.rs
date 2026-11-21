//! Validates a FHIR resource against a profile (a `StructureDefinition` with
//! `derivation = constraint`).
//!
//! A profile is compiled once into a tree the validator walks directly, and
//! that compiled form, the profiles it refers to and the value sets its slices
//! bind are cached for as long as the [`CanonicalResolver`] keeps handing out
//! the same resources. Validating a resource is then a synchronous walk over
//! the value with no path parsing and no resolver calls.
use haste_fhir_client::canonical_resolver::CanonicalResolver;
use haste_fhir_model::r4::generated::{
    resources::{OperationOutcome, Resource, ResourceType, StructureDefinition},
    terminology::{IssueType, TypeDerivationRule},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_reflect::MetaValue;
use std::sync::Arc;

mod compile;
mod graph;
mod slicing;
mod validate;
mod validators;

pub struct FHIRProfileArguments<Resolver: CanonicalResolver> {
    resolver: Arc<Resolver>,
}

impl<Resolver: CanonicalResolver> FHIRProfileArguments<Resolver> {
    pub fn new(resolver: Arc<Resolver>) -> Self {
        Self { resolver }
    }
}

#[derive(Clone)]
pub struct FHIRProfileCTX<'a, Resolver: CanonicalResolver> {
    resolver: Arc<Resolver>,
    profile: Arc<Resource>,
    root: &'a dyn MetaValue,
}

impl<'a, Resolver: CanonicalResolver> FHIRProfileCTX<'a, Resolver> {
    /// Creates a new FHIR profile context.
    ///
    /// # Errors
    ///
    /// Returns [`OperationOutcomeError`] if `profile` is not a
    /// [`StructureDefinition`] resource.
    pub fn new(
        resolver: Arc<Resolver>,
        profile: Arc<Resource>,
        root: &'a dyn MetaValue,
    ) -> Result<Self, OperationOutcomeError> {
        match &*profile {
            Resource::StructureDefinition(_profile) => Ok(Self {
                resolver,
                profile,
                root,
            }),
            _ => Err(OperationOutcomeError::error(
                IssueType::invalid(),
                "Profile resource must be a StructureDefinition".to_string(),
            )),
        }
    }

    /// Returns the [`StructureDefinition`] associated with this profile context.
    ///
    /// # Panics
    ///
    /// Panics if the profile is not a [`Resource::StructureDefinition`].
    #[must_use]
    pub fn profile(&'a self) -> &'a StructureDefinition {
        match self.profile.as_ref() {
            Resource::StructureDefinition(sd) => sd,
            _ => panic!(
                "Invalid state for profile ctx, profile field must be a StructureDefinition."
            ),
        }
    }
}

/// Validates the profile represented by the given context.
///
/// # Errors
///
/// Returns [`OperationOutcomeError`] if:
/// - validating an element fails; or
/// - the profile does not have a derivation of `constraint`.
pub async fn validate_profile(
    ctx: Arc<FHIRProfileCTX<'_, impl CanonicalResolver>>,
) -> Result<OperationOutcome, OperationOutcomeError> {
    if ctx.profile().derivation.as_ref() != Some(&TypeDerivationRule::constraint()) {
        return Err(OperationOutcomeError::error(
            IssueType::invalid(),
            "Only profiles with derivation 'constraint' are supported".to_string(),
        ));
    }
    let graph = graph::graph_for(ctx.resolver.as_ref(), &ctx.profile).await?;
    Ok(OperationOutcome {
        issue: validate::validate(&graph, ctx.root)?,
        ..Default::default()
    })
}

/// Validates a value against the profile identified by its canonical URL.
///
/// # Errors
///
/// Returns [`OperationOutcomeError`] if resolving the profile fails, if the
/// resolved resource is not a supported profile, or if profile validation fails.
pub async fn validate_profile_by_url(
    args: FHIRProfileArguments<impl CanonicalResolver>,
    canonical_url: &str,
    value: &dyn MetaValue,
) -> Result<OperationOutcome, OperationOutcomeError> {
    let Some(profile) = args
        .resolver
        .resolve(ResourceType::StructureDefinition, canonical_url)
        .await?
    else {
        tracing::warn!("Profile with url '{canonical_url}' not found");
        return Ok(OperationOutcome::default());
    };

    let ctx = Arc::new(FHIRProfileCTX::new(args.resolver.clone(), profile, value)?);

    validate_profile(ctx).await
}
