use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, LazyLock},
};

use haste_codegen::{
    traversal::ele_index_to_child_indices,
    utilities::extract::{Max, cardinality, field_name},
};
use haste_fhir_client::canonical_resolver::CanonicalResolver;
use haste_fhir_model::r4::generated::{
    resources::{OperationOutcomeIssue, ResourceType, StructureDefinition},
    terminology::{DiscriminatorType, IssueSeverity, IssueType},
    types::{ElementDefinition, ElementDefinitionSlicingDiscriminator},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_pointer::Path;
use haste_reflect::MetaValue;

use crate::{
    FHIRProfileCTX,
    element::{outcome_issue, validate_singular_element},
    utilities::{self, convert_discriminator_to_path},
    validators::{fixed_value::is_equal, pattern::validate_pattern},
};

fn is_slice(element: &ElementDefinition) -> bool {
    element.slicing.is_some()
}

#[derive(Debug)]
pub struct SlicingDescriptor {
    /// The index of the element definition that contains the discriminator.
    pub discriminator: usize,
    /// The indices of the slice element definitions that belong to the discriminator. The discriminator element is not included in this list.
    pub slices: Vec<usize>,
}

/// Return child elements that are slice element definitions.
pub fn get_slice_descriptors(
    elements: &[Box<ElementDefinition>],
    children: &Vec<usize>,
) -> Result<Vec<SlicingDescriptor>, OperationOutcomeError> {
    let mut i = 0;

    let mut slice_indices = vec![];

    while i < children.len() {
        let child_index = children[i];
        let element = &elements[child_index];
        i += 1;

        if is_slice(element.as_ref()) {
            let mut slice_index = SlicingDescriptor {
                discriminator: child_index,
                slices: vec![],
            };

            // SliceName indicates that it's a part of the slice (upper discriminator).
            // So we keep adding to the slice until we find an element that doesn't have a sliceName or we run out of children.
            while i < children.len()
                && elements[children[i]]
                    .sliceName
                    .as_ref()
                    .and_then(|v| v.value.as_ref())
                    .is_some()
            {
                slice_index.slices.push(children[i]);
                i += 1;
            }

            slice_indices.push(slice_index);
        }
    }

    Ok(slice_indices)
}

struct FoundDiscriminator<'a, Resolver: CanonicalResolver> {
    ctx: Arc<FHIRProfileCTX<'a, Resolver>>,

    discriminator_element_index: usize,
}

fn join_paths(parent: &str, child: &str) -> String {
    if parent.is_empty() {
        child.to_string()
    } else if child.is_empty() {
        parent.to_string()
    } else {
        format!("{}.{}", parent, child)
    }
}

/// The discriminator element specifies a path from which to compare with.
/// To know how split should be done though we need the constant pattern etc... from that path.
/// For example Extension.url could be the discriminator, but
/// We need to pull from for example https://build.fhir.org/ig/HL7/US-Core/StructureDefinition-us-core-race.html
///  the actual value of the pattern to know how to split the slice. Which would be "http://hl7.org/fhir/us/core/StructureDefinition/us-core-race"
async fn find_element_definition_for_discriminator<'a, Resolver: CanonicalResolver>(
    ctx: Arc<FHIRProfileCTX<'a, Resolver>>,
    search_for_path: &str,
    current_index: usize,
    parent_path: Option<&str>,
) -> Result<Option<FoundDiscriminator<'a, Resolver>>, OperationOutcomeError> {
    let element_to_check = ctx
        .profile()
        .snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.element.get(current_index))
        .ok_or_else(|| {
            OperationOutcomeError::error(
                IssueType::Exception(None),
                format!("Invalid element index: {}", current_index),
            )
        })?;
    let element_path = element_to_check
        .path
        .value
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("");

    let current_element_path = if let Some(parent_path) = parent_path {
        join_paths(parent_path, utilities::remove_type_on_path(element_path))
    } else {
        utilities::remove_type_on_path(element_path).to_string()
    };

    if current_element_path == search_for_path {
        return Ok(Some(FoundDiscriminator {
            ctx: ctx.clone(),
            discriminator_element_index: current_index,
        }));
    }

    if search_for_path.starts_with(&current_element_path) {
        if let Some(profiles) = element_to_check.type_.as_ref().map(|types_| {
            types_
                .iter()
                .filter_map(|t| t.profile.as_ref())
                .flatten()
                .collect::<Vec<_>>()
        }) && !profiles.is_empty()
        {
            for profile in profiles.iter() {
                if let Some(canonical) = profile.value.as_ref().map(|c| c.as_str()) {
                    let resolved_profile = ctx
                        .resolver
                        .resolve(ResourceType::StructureDefinition, canonical)
                        .await?
                        .ok_or_else(|| {
                            OperationOutcomeError::error(
                                IssueType::Exception(None),
                                format!("Failed to resolve profile canonical: {}", canonical),
                            )
                        })?;

                    let found_discriminator = Box::pin(find_element_definition_for_discriminator(
                        Arc::new(FHIRProfileCTX::new(
                            ctx.resolver.clone(),
                            resolved_profile,
                            ctx.root,
                        )?),
                        search_for_path,
                        0,
                        Some(&current_element_path),
                    ))
                    .await?;

                    if let Some(v) = found_discriminator {
                        return Ok(Some(v));
                    }
                }
            }
        }

        let default = vec![];

        let child_indices = ele_index_to_child_indices(
            ctx.profile()
                .snapshot
                .as_ref()
                .map(|s| s.element.as_ref())
                .unwrap_or(&default),
            current_index,
        )
        .map_err(|err| OperationOutcomeError::error(IssueType::Exception(None), err))?;

        for child_index in child_indices {
            let found_discriminator = Box::pin(find_element_definition_for_discriminator(
                ctx.clone(),
                search_for_path,
                child_index,
                Some(&current_element_path),
            ))
            .await?;

            if let Some(v) = found_discriminator {
                return Ok(Some(v));
            }
        }
    };

    Ok(None)
}

/// Returns all the slice locs that are relevant to the given discriminator.
fn get_slice_value_locs(
    discriminator_element: &ElementDefinition,
    value: &dyn MetaValue,
    value_path: &Path,
) -> Result<Vec<Path>, OperationOutcomeError> {
    let field = field_name(
        discriminator_element
            .path
            .value
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or(""),
    );

    let slice_path = value_path.descend(&field);

    let Some(v) = slice_path.get(value) else {
        return Ok(vec![]);
    };

    if v.is_many() {
        Ok(v.flatten()
            .iter()
            .enumerate()
            .map(|(i, _)| slice_path.descend(&format!("{}", i)))
            .collect())
    } else {
        Ok(vec![slice_path])
    }
}

static FP_ENGINE: LazyLock<haste_fhirpath::FPEngine> =
    LazyLock::new(|| haste_fhirpath::FPEngine::new());

async fn is_conformant_to_slice_descriptor(
    discriminator: &ElementDefinitionSlicingDiscriminator,
    slice_value_element_definition: &ElementDefinition,
    root: &dyn MetaValue,
    path: &Path,
) -> Result<bool, OperationOutcomeError> {
    let value = path.get(root).ok_or_else(|| {
        OperationOutcomeError::error(
            IssueType::Invalid(None),
            "Value for discriminator not found at path".to_string(),
        )
    })?;
    let values = FP_ENGINE
        .evaluate(
            discriminator
                .path
                .value
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or("$this"),
            vec![value],
        )
        .await
        .map_err(|err| {
            OperationOutcomeError::error(
                IssueType::Exception(None),
                format!(
                    "Failed to evaluate FHIRPath expression for discriminator: {}",
                    err
                ),
            )
        })?;
    let values = values.iter().collect::<Vec<_>>();

    match discriminator.type_.as_ref() {
        DiscriminatorType::Exists(_) => Ok(values.len() > 0),
        DiscriminatorType::Pattern(_) => {
            let pattern = slice_value_element_definition.pattern.as_ref().ok_or_else(|| OperationOutcomeError::error(
                IssueType::Invalid(None),
                "Slice value element definition must have a pattern for pattern discriminator".to_string(),
            ))?;

            for value in values.iter() {
                if validate_pattern(*value, pattern)? {
                    return Ok(true);
                }
            }

            return Ok(false);
        }
        DiscriminatorType::Profile(_) => Err(OperationOutcomeError::error(
            IssueType::NotSupported(None),
            "Profile discriminator type is not supported".to_string(),
        )),
        DiscriminatorType::Type(_) => {
            let expected_types =
                slice_value_element_definition
                    .type_
                    .as_ref()
                    .ok_or_else(|| {
                        OperationOutcomeError::error(
                            IssueType::Invalid(None),
                            "Slice value element definition must have types for type discriminator"
                                .to_string(),
                        )
                    })?;
            let types = values.iter().map(|v| v.typename()).collect::<HashSet<_>>();

            let result = expected_types.iter().find(|t| {
                if let Some(type_name) = t.code.value.as_ref().map(|c| c.as_str()) {
                    types.contains(type_name)
                } else {
                    false
                }
            });

            Ok(result.is_some())
        }
        DiscriminatorType::Value(_) => {
            let fixed_value  = slice_value_element_definition.fixed.as_ref().ok_or_else(|| OperationOutcomeError::error(
                IssueType::Invalid(None),
                "Slice value element definition must have a fixed value for value discriminator".to_string(),
            ))?;

            for value in values.iter() {
                if is_equal(*value, fixed_value)? {
                    return Ok(true);
                }
            }
            return Ok(false);
        }
        DiscriminatorType::Null(_) => Err(OperationOutcomeError::error(
            IssueType::NotSupported(None),
            "Null discriminator type is not supported".to_string(),
        )),
    }
}

struct SplitSlicing(HashMap<usize, Vec<Path>>);

/// Splits the given values into slices according to the discriminator.
async fn split_slicing<'a>(
    ctx: Arc<FHIRProfileCTX<'a, impl CanonicalResolver>>,
    slicing_descriptor: &SlicingDescriptor,
    value: &dyn MetaValue,
    mut locs: Vec<Path>,
) -> Result<SplitSlicing, OperationOutcomeError> {
    let mut slices_split = SplitSlicing(HashMap::new());
    let discriminator_element_definition =
        get_element(ctx.profile(), slicing_descriptor.discriminator)?;
    let discriminators = discriminator_element_definition
        .slicing
        .as_ref()
        .and_then(|s| s.discriminator.as_ref())
        .ok_or_else(|| {
            OperationOutcomeError::error(
                IssueType::Invalid(None),
                "Invalid slicing discriminator configuration".to_string(),
            )
        })?;

    let discriminator_element_paths = discriminators
        .iter()
        .map(|d| convert_discriminator_to_path(discriminator_element_definition, d))
        .collect::<Result<Vec<_>, _>>()?;

    for slice_index in &slicing_descriptor.slices {
        for (discriminator_element_index, discriminator_element_path) in
            discriminator_element_paths.iter().enumerate()
        {
            let discriminator = &discriminators[discriminator_element_index];
            let Some(slice_descriminator_value_definition) =
                find_element_definition_for_discriminator(
                    ctx.clone(),
                    discriminator_element_path,
                    *slice_index,
                    None,
                )
                .await?
            else {
                return Err(OperationOutcomeError::error(
                    IssueType::Invalid(None),
                    format!(
                        "Failed to find element definition for discriminator path '{}'",
                        discriminator_element_path
                    ),
                ));
            };

            let mut remainder_locs = vec![];
            let mut slice_locations = vec![];
            for loc in locs.into_iter() {
                if is_conformant_to_slice_descriptor(
                    discriminator,
                    get_element(
                        slice_descriminator_value_definition.ctx.profile(),
                        slice_descriminator_value_definition.discriminator_element_index,
                    )?,
                    value,
                    &loc,
                )
                .await?
                {
                    slice_locations.push(loc.clone());
                } else {
                    remainder_locs.push(loc.clone());
                }
            }

            slices_split.0.insert(*slice_index, slice_locations);

            locs = remainder_locs;
        }
    }

    Ok(slices_split)
}

fn get_element<'a>(
    profile: &'a StructureDefinition,
    element_index: usize,
) -> Result<&'a Box<ElementDefinition>, OperationOutcomeError> {
    let element = profile
        .snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.element.get(element_index))
        .ok_or_else(|| {
            OperationOutcomeError::error(
                IssueType::Exception(None),
                format!("Invalid slicing discriminator index: {}", element_index),
            )
        })?;

    Ok(element)
}

fn validate_slice_cardinality(
    slice_element_definition: &ElementDefinition,
    slice_locs: &Vec<Path>,
) -> Result<Vec<OperationOutcomeIssue>, OperationOutcomeError> {
    let (min, max) = cardinality(slice_element_definition);

    let mut issues = vec![];

    if slice_locs.len() < min as usize {
        issues.push(outcome_issue(
            slice_locs.first().unwrap_or(&Path::new()),
            IssueSeverity::Error(None),
            IssueType::Value(None),
            format!(
                "Cardinality too low: expected at least '{}', found '{}'",
                min,
                slice_locs.len()
            ),
        ));
    }

    match max {
        Max::Fixed(fixed_max) => {
            if slice_locs.len() > fixed_max as usize {
                issues.push(outcome_issue(
                    slice_locs.first().unwrap_or(&Path::new()),
                    IssueSeverity::Error(None),
                    IssueType::Value(None),
                    format!(
                        "Cardinality too high: expected at most '{}', found '{}'",
                        fixed_max,
                        slice_locs.len()
                    ),
                ));
            }
        }
        // Do nothing if max is unlimited, as there is no upper bound to violate.
        Max::Unlimited => {}
    }

    Ok(issues)
}

pub async fn validate_slicing_descriptor<'a>(
    ctx: Arc<FHIRProfileCTX<'a, impl CanonicalResolver>>,
    slicing_descriptor: &SlicingDescriptor,
    value: &dyn MetaValue,
    value_path: &Path,
) -> Result<Vec<OperationOutcomeIssue>, OperationOutcomeError> {
    let discriminator_element = get_element(ctx.profile(), slicing_descriptor.discriminator)?;
    let all_slice_locs = get_slice_value_locs(discriminator_element, value, value_path)?;
    let split_slices =
        split_slicing(ctx.clone(), slicing_descriptor, value, all_slice_locs).await?;
    let mut issues = vec![];
    let elements_pointer = Path::new().descend("snapshot").descend("element");

    for slice in slicing_descriptor.slices.iter() {
        let slice_locs = split_slices.0.get(slice).ok_or_else(|| {
            OperationOutcomeError::error(
                IssueType::Exception(None),
                format!("Missing slice locations for slice index: {}", slice),
            )
        })?;

        let slice_element_definition = get_element(ctx.profile(), *slice)?;

        issues.extend(validate_slice_cardinality(
            slice_element_definition,
            slice_locs,
        )?);

        for slice_loc in slice_locs.iter() {
            issues.extend(
                validate_singular_element(
                    ctx.clone(),
                    &elements_pointer.descend(&format!("{}", slice)),
                    slice_loc,
                )
                .await?,
            );
        }
    }

    Ok(issues)
}
