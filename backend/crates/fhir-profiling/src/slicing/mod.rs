use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, LazyLock},
};

use haste_codegen::{
    traversal::ele_index_to_child_indices,
    utilities::extract::{Max, cardinality, field_name},
};
use haste_fhir_client::canonical_resolver::CanonicalResolver;
use haste_fhir_model::r4::{
    conversion::downcast_string,
    generated::{
        resources::{
            OperationOutcome, OperationOutcomeIssue, Resource, ResourceType, StructureDefinition,
            ValueSetComposeInclude,
        },
        terminology::{BindingStrength, BoundCode, DiscriminatorType, IssueSeverity, IssueType},
        types::{
            CodeableConcept, Coding, ElementDefinition, ElementDefinitionSlicingDiscriminator,
        },
    },
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
    elements: &[ElementDefinition],
    children: &[usize],
) -> Vec<SlicingDescriptor> {
    let mut i = 0;

    let mut slice_indices = Vec::new();

    while i < children.len() {
        let child_index = children[i];
        let element = &elements[child_index];
        i += 1;

        if is_slice(element) {
            let mut slice_index = SlicingDescriptor {
                discriminator: child_index,
                slices: Vec::new(),
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

    slice_indices
}

struct FoundDiscriminator<'a, Resolver: CanonicalResolver> {
    ctx: Arc<FHIRProfileCTX<'a, Resolver>>,

    discriminator_element_index: usize,

    /// For a `value` or `pattern` discriminator on a slice defined by its binding alone, the codes
    /// it allows.
    binding: Option<ValueSetCodes>,
}

/// The discriminator element specifies a path from which to compare with.
/// To know how split should be done though we need the constant pattern etc... from that path.
/// For example Extension.url could be the discriminator, but
/// We need to pull from for example
/// <https://build.fhir.org/ig/HL7/US-Core/StructureDefinition-us-core-race.html>
///  the actual value of the pattern to know how to split the slice. Which would
///  be <http://hl7.org/fhir/us/core/StructureDefinition/us-core-race>
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
                IssueType::exception(),
                format!("Invalid element index: {current_index}"),
            )
        })?;

    let element_path = element_to_check.path.value.as_deref().unwrap_or("");

    let current_element_path = if let Some(parent_path) = parent_path {
        utilities::join_paths(
            parent_path,
            utilities::get_element_field(element_path).unwrap_or(""),
        )
    } else {
        utilities::get_element_field(element_path)
            .unwrap_or("")
            .to_string()
    };

    if current_element_path == search_for_path {
        return Ok(Some(FoundDiscriminator {
            ctx: ctx.clone(),
            discriminator_element_index: current_index,
            binding: None,
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
            for profile in &profiles {
                if let Some(canonical) = profile.value.as_deref() {
                    let resolved_profile = ctx
                        .resolver
                        .resolve(ResourceType::StructureDefinition, canonical)
                        .await?
                        .ok_or_else(|| {
                            OperationOutcomeError::error(
                                IssueType::exception(),
                                format!("Failed to resolve profile canonical: {canonical}"),
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

        let default = Vec::new();

        let child_indices = ele_index_to_child_indices(
            ctx.profile()
                .snapshot
                .as_ref()
                .map_or(&default, |s| s.element.as_ref()),
            current_index,
        )
        .map_err(|err| OperationOutcomeError::error(IssueType::exception(), err))?;

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
    }

    Ok(None)
}

/// Returns all the slice locs that are relevant to the given discriminator.
fn get_slice_value_locs(
    ctx: &Arc<FHIRProfileCTX<'_, impl CanonicalResolver>>,
    discriminator_element: &ElementDefinition,
    value_path: &Path,
) -> Vec<Path> {
    let field = field_name(discriminator_element.path.value.as_deref().unwrap_or(""));

    let slice_path = value_path.descend(&field);
    let Some(v) = slice_path.get(ctx.root) else {
        return Vec::new();
    };

    if v.is_many() {
        v.flatten()
            .iter()
            .enumerate()
            .map(|(i, _)| slice_path.descend(&i.to_string()))
            .collect()
    } else {
        vec![slice_path]
    }
}

static FP_ENGINE: LazyLock<haste_fhirpath::FPEngine> = LazyLock::new(haste_fhirpath::FPEngine::new);

async fn is_conformant_to_slice_descriptor(
    discriminator: &ElementDefinitionSlicingDiscriminator,
    slice_value_element_definition: &ElementDefinition,
    binding: Option<&ValueSetCodes>,
    root: &dyn MetaValue,
    path: &Path,
) -> Result<bool, OperationOutcomeError> {
    let value = path.get(root).ok_or_else(|| {
        OperationOutcomeError::error(
            IssueType::invalid(),
            "Value for discriminator not found at path".to_string(),
        )
    })?;

    let values = FP_ENGINE
        .evaluate(
            discriminator.path.value.as_deref().unwrap_or("$this"),
            vec![value],
        )
        .await
        .map_err(|err| {
            OperationOutcomeError::error(
                IssueType::exception(),
                format!("Failed to evaluate FHIRPath expression for discriminator: {err}"),
            )
        })?;

    let values = values.iter().collect::<Vec<_>>();

    match &discriminator.type_ {
        discriminator_type if discriminator_type == &DiscriminatorType::exists() => {
            Ok(!values.is_empty())
        }
        discriminator_type if is_value_discriminator(discriminator_type) => {
            is_conformant_to_value_discriminator(&values, slice_value_element_definition, binding)
        }
        discriminator_type if discriminator_type == &DiscriminatorType::profile() => {
            Err(OperationOutcomeError::error(
                IssueType::not_supported(),
                "Profile discriminator type is not supported".to_string(),
            ))
        }
        discriminator_type if discriminator_type == &DiscriminatorType::type_() => {
            is_conformant_to_type_discriminator(&values, slice_value_element_definition)
        }
        discriminator_type if discriminator_type == &DiscriminatorType::null() => {
            Err(OperationOutcomeError::error(
                IssueType::not_supported(),
                "Null discriminator type is not supported".to_string(),
            ))
        }
        _ => Err(OperationOutcomeError::error(
            IssueType::not_supported(),
            "Unsupported discriminator type".to_string(),
        )),
    }
}

/// A `value` or `pattern` discriminator. FHIR R4 gives both the same rule: each slice defines the
/// discriminating element with `fixed[x]`, `pattern[x]` or a required binding to a value set that
/// lists its codes (<https://hl7.org/fhir/R4/profiling.html#discriminator>). R5 deprecates
/// `pattern` as meaning the same as `value`
/// (<https://hl7.org/fhir/R5/profiling.html#discriminator>). US Core 7.0.0's category slices use
/// `pattern` with only a required binding.
fn is_value_discriminator(discriminator_type: &BoundCode<DiscriminatorType>) -> bool {
    discriminator_type == &DiscriminatorType::value()
        || discriminator_type == &DiscriminatorType::pattern()
}

fn is_conformant_to_type_discriminator(
    values: &[&dyn MetaValue],
    element: &ElementDefinition,
) -> Result<bool, OperationOutcomeError> {
    let expected_types = element.type_.as_ref().ok_or_else(|| {
        OperationOutcomeError::error(
            IssueType::invalid(),
            "Slice value element definition must have types for type discriminator".to_string(),
        )
    })?;

    let types = values
        .iter()
        .map(|value| value.fhir_type())
        .collect::<HashSet<_>>();

    let result = expected_types.iter().find(|type_| {
        if let Some(type_name) = type_.code.value.as_deref() {
            types.contains(type_name)
        } else {
            false
        }
    });

    Ok(result.is_some())
}

fn is_conformant_to_value_discriminator(
    values: &[&dyn MetaValue],
    element: &ElementDefinition,
    binding: Option<&ValueSetCodes>,
) -> Result<bool, OperationOutcomeError> {
    if let Some(fixed_value) = element.fixed.as_ref() {
        for value in values {
            if is_equal(*value, fixed_value)? {
                return Ok(true);
            }
        }

        return Ok(false);
    }

    if let Some(pattern) = element.pattern.as_ref() {
        for value in values {
            if validate_pattern(*value, pattern)? {
                return Ok(true);
            }
        }

        return Ok(false);
    }

    if let Some(binding) = binding {
        return Ok(values.iter().any(|value| binding.matches(*value)));
    }

    Err(OperationOutcomeError::error(
        IssueType::invalid(),
        "Slice value element definition must have a fixed value, a pattern or a binding for value discriminator"
            .to_string(),
    ))
}

/// Bounds how deeply value sets may include each other, so a cycle ends.
const MAX_VALUE_SET_DEPTH: u8 = 8;

/// The codes a value set's `compose` allows, read without expanding it. A listed concept matches
/// on its system and code, a whole system on the system alone. Filters are not evaluated, so an
/// include or exclude with one allows no codes.
#[derive(Debug, Default)]
struct ValueSetCodes {
    include: Vec<ComposeRule>,
    exclude: Vec<ComposeRule>,
}

/// One `compose.include` or `compose.exclude`: a code is in it when it meets every part given.
#[derive(Debug)]
struct ComposeRule {
    system: Option<String>,
    /// `None` for every code in `system`.
    codes: Option<HashSet<String>>,
    value_sets: Vec<ValueSetCodes>,
}

impl ComposeRule {
    /// `system` is `None` for a bare `code`, whose system the binding implies.
    fn contains(&self, system: Option<&str>, code: &str) -> bool {
        (system.is_none() || self.system.is_none() || system == self.system.as_deref())
            && self.codes.as_ref().is_none_or(|codes| codes.contains(code))
            && self
                .value_sets
                .iter()
                .all(|value_set| value_set.contains(system, code))
    }
}

impl ValueSetCodes {
    fn contains(&self, system: Option<&str>, code: &str) -> bool {
        self.include.iter().any(|rule| rule.contains(system, code))
            && !self.exclude.iter().any(|rule| rule.contains(system, code))
    }

    /// A `Coding` matches on its system and code, a `CodeableConcept` when any of its codings does,
    /// and a `code` or `string` on the code.
    fn matches(&self, value: &dyn MetaValue) -> bool {
        let coding_matches = |coding: &Coding| {
            let system = coding
                .system
                .as_ref()
                .and_then(|system| system.value.as_deref());
            let code = coding.code.as_ref().and_then(|code| code.value.as_deref());
            system
                .zip(code)
                .is_some_and(|(system, code)| self.contains(Some(system), code))
        };
        match value.fhir_type() {
            "Coding" => downcast::<Coding>(value).is_some_and(coding_matches),
            "CodeableConcept" => downcast::<CodeableConcept>(value)
                .and_then(|concept| concept.coding.as_ref())
                .is_some_and(|codings| codings.iter().any(coding_matches)),
            _ => downcast_string(value).is_ok_and(|code| self.contains(None, &code)),
        }
    }
}

/// `value` as a `T`, which a singular field holds in a `Box`.
fn downcast<T: 'static>(value: &dyn MetaValue) -> Option<&T> {
    let value = value.as_any();
    value
        .downcast_ref()
        .or_else(|| value.downcast_ref::<Box<T>>().map(AsRef::as_ref))
}

/// `None` when the value set at `url`, or one it includes, cannot be resolved.
async fn value_set_codes(
    resolver: &impl CanonicalResolver,
    url: &str,
    depth: u8,
) -> Result<Option<ValueSetCodes>, OperationOutcomeError> {
    let Some(depth) = depth.checked_sub(1) else {
        return Ok(None);
    };
    let resource = resolver.resolve(ResourceType::ValueSet, url).await?;
    let Some(Resource::ValueSet(value_set)) = resource.as_deref() else {
        return Ok(None);
    };
    let Some(compose) = &value_set.compose else {
        return Ok(Some(ValueSetCodes::default()));
    };
    let exclude = compose.exclude.as_deref().unwrap_or_default();
    let include = compose_rules(resolver, &compose.include, depth).await?;
    let exclude = compose_rules(resolver, exclude, depth).await?;
    Ok(include
        .zip(exclude)
        .map(|(include, exclude)| ValueSetCodes { include, exclude }))
}

async fn compose_rules(
    resolver: &impl CanonicalResolver,
    rules: &[ValueSetComposeInclude],
    depth: u8,
) -> Result<Option<Vec<ComposeRule>>, OperationOutcomeError> {
    let mut compose_rules = Vec::with_capacity(rules.len());
    for rule in rules {
        let mut value_sets = Vec::new();
        for url in rule
            .valueSet
            .iter()
            .flatten()
            .filter_map(|url| url.value.as_deref())
        {
            let Some(value_set) = Box::pin(value_set_codes(resolver, url, depth)).await? else {
                return Ok(None);
            };
            value_sets.push(value_set);
        }
        let system = rule.system.as_ref().and_then(|system| system.value.clone());
        let codes = if rule.filter.is_some() || (system.is_none() && value_sets.is_empty()) {
            Some(HashSet::new())
        } else {
            rule.concept.as_ref().map(|concepts| {
                concepts
                    .iter()
                    .filter_map(|concept| concept.code.value.clone())
                    .collect()
            })
        };
        compose_rules.push(ComposeRule {
            system,
            codes,
            value_sets,
        });
    }
    Ok(Some(compose_rules))
}

/// For a `value` or `pattern` discriminator on a slice defined by its binding alone (no
/// `fixed[x]` or `pattern[x]`), the codes that binding allows. Only a required binding
/// discriminates; any other allows none. A value set that cannot be resolved allows none, or is
/// reported when the slice is mandatory.
async fn binding_codes(
    ctx: &FHIRProfileCTX<'_, impl CanonicalResolver>,
    slice_index: usize,
    discriminator: &ElementDefinitionSlicingDiscriminator,
    element: &ElementDefinition,
    loc: &Path,
) -> Result<Option<ValueSetCodes>, OperationOutcomeError> {
    let Some(binding) = element.binding.as_ref().filter(|_| {
        is_value_discriminator(&discriminator.type_)
            && element.fixed.is_none()
            && element.pattern.is_none()
    }) else {
        return Ok(None);
    };
    let Some(url) = binding
        .valueSet
        .as_ref()
        .and_then(|url| url.value.as_deref())
        .filter(|_| binding.strength == BindingStrength::required())
    else {
        return Ok(Some(ValueSetCodes::default()));
    };
    if let Some(codes) = value_set_codes(ctx.resolver.as_ref(), url, MAX_VALUE_SET_DEPTH).await? {
        return Ok(Some(codes));
    }

    let profile = ctx.profile();
    let slice = get_element(profile, slice_index)?;
    if cardinality(slice).0 == 0 {
        return Ok(Some(ValueSetCodes::default()));
    }
    Err(OperationOutcomeError::unprocessable(OperationOutcome {
        issue: vec![outcome_issue(
            loc,
            IssueSeverity::error(),
            IssueType::not_found(),
            format!(
                "Profile: '{}' Element: '{}' The slice's ValueSet '{url}' could not be resolved",
                profile.id.as_deref().unwrap_or("unknown"),
                slice.id.as_deref().unwrap_or("unknown"),
            ),
        )],
        ..Default::default()
    }))
}

#[derive(Debug)]
struct SplitSlicing(HashMap<usize, Vec<Path>>);

async fn find_discriminator_definition<'a, Resolver: CanonicalResolver>(
    ctx: &Arc<FHIRProfileCTX<'a, Resolver>>,
    slice_index: usize,
    discriminator_element_path: &str,
    parent_path: Option<&str>,
) -> Result<FoundDiscriminator<'a, Resolver>, OperationOutcomeError> {
    find_element_definition_for_discriminator(
        ctx.clone(),
        discriminator_element_path,
        slice_index,
        parent_path,
    )
    .await?
    .ok_or_else(|| {
        OperationOutcomeError::error(
            IssueType::invalid(),
            format!(
                "'{}' Failed to find element definition for discriminator path '{}'",
                ctx.profile().id.as_deref().unwrap_or("unknown"),
                discriminator_element_path
            ),
        )
    })
}

/// `definitions` holds, per discriminator, the slice's element definition for it.
/// That definition depends only on the slice, not on `loc`, so it is looked up for
/// the first value that reaches the discriminator and reused for the other values.
/// Looking it up on first use rather than up front keeps a failed lookup reported
/// at the same point as before.
async fn is_conformant_to_discriminators<'a, Resolver: CanonicalResolver>(
    ctx: &Arc<FHIRProfileCTX<'a, Resolver>>,
    slice_index: usize,
    discriminators: &[ElementDefinitionSlicingDiscriminator],
    discriminator_element_paths: &[String],
    parent_path: Option<&str>,
    definitions: &mut [Option<FoundDiscriminator<'a, Resolver>>],
    loc: &Path,
) -> Result<bool, OperationOutcomeError> {
    for (discriminator_element_index, discriminator_element_path) in
        discriminator_element_paths.iter().enumerate()
    {
        let discriminator = &discriminators[discriminator_element_index];
        let slice_descriminator_value_definition =
            match &mut definitions[discriminator_element_index] {
                Some(definition) => definition,
                slot => {
                    let mut definition = find_discriminator_definition(
                        ctx,
                        slice_index,
                        discriminator_element_path,
                        parent_path,
                    )
                    .await?;
                    let element = get_element(
                        definition.ctx.profile(),
                        definition.discriminator_element_index,
                    )?;
                    definition.binding =
                        binding_codes(ctx, slice_index, discriminator, element, loc).await?;
                    slot.insert(definition)
                }
            };

        if !is_conformant_to_slice_descriptor(
            discriminator,
            get_element(
                slice_descriminator_value_definition.ctx.profile(),
                slice_descriminator_value_definition.discriminator_element_index,
            )?,
            slice_descriminator_value_definition.binding.as_ref(),
            ctx.root,
            loc,
        )
        .await?
        {
            return Ok(false);
        }
    }

    Ok(true)
}

/// Splits the given values into slices according to the discriminator.
async fn split_slicing(
    ctx: Arc<FHIRProfileCTX<'_, impl CanonicalResolver>>,
    slicing_descriptor: &SlicingDescriptor,
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
                IssueType::invalid(),
                "Invalid slicing discriminator configuration".to_string(),
            )
        })?;

    // Set the initial parent as the discriminator IE if the discriminator is Observation.coding.code than the parent_path
    // should be coding because the discriminator path is relative to the element with the discriminator.
    // So we need to remove the type on the path and ascend to get the correct path to check on the instance data.
    let parent_path = discriminator_element_definition
        .path
        .value
        .as_deref()
        .and_then(utilities::ascend_element_path)
        .map(utilities::remove_type_on_path);

    let discriminator_element_paths = discriminators
        .iter()
        .map(|d| convert_discriminator_to_path(discriminator_element_definition, d))
        .collect::<Result<Vec<_>, _>>()?;

    for slice_index in &slicing_descriptor.slices {
        let mut remainder_locs = vec![];
        let mut slice_locations = vec![];
        let mut definitions: Vec<_> = discriminators.iter().map(|_| None).collect();

        for loc in &locs {
            if is_conformant_to_discriminators(
                &ctx,
                *slice_index,
                discriminators,
                &discriminator_element_paths,
                parent_path,
                &mut definitions,
                loc,
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

    Ok(slices_split)
}

fn get_element(
    profile: &StructureDefinition,
    element_index: usize,
) -> Result<&ElementDefinition, OperationOutcomeError> {
    let element = profile
        .snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.element.get(element_index))
        .ok_or_else(|| {
            OperationOutcomeError::error(
                IssueType::exception(),
                format!("Invalid slicing discriminator index: {element_index}"),
            )
        })?;

    Ok(element)
}

fn validate_slice_cardinality(
    profile: &StructureDefinition,
    slice_element_definition: &ElementDefinition,
    slice_locs: &[Path],
) -> Vec<OperationOutcomeIssue> {
    let (min, max) = cardinality(slice_element_definition);

    let mut issues = Vec::new();

    if (slice_locs.len() as u64) < min {
        issues.push(outcome_issue(
            slice_locs.first().unwrap_or(&Path::new()),
            IssueSeverity::error(),
            IssueType::value(),
            format!(
                 "Profile: '{}' Element: '{}' Minimum number of required values not met expected at least '{}', found '{}'",
                profile.id.as_deref().unwrap_or("unknown"),
                slice_element_definition.id.as_deref().unwrap_or("unknown"),
                min,
                slice_locs.len()
            ),
        ));
    }

    match max {
        Max::Fixed(fixed_max) => {
            if (slice_locs.len() as u64) > fixed_max {
                issues.push(outcome_issue(
                    slice_locs.first().unwrap_or(&Path::new()),
                    IssueSeverity::error(),
                    IssueType::value(),
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

    issues
}

pub async fn validate_slicing_descriptor(
    ctx: Arc<FHIRProfileCTX<'_, impl CanonicalResolver>>,
    slicing_descriptor: &SlicingDescriptor,
    value_path: &Path,
) -> Result<Vec<OperationOutcomeIssue>, OperationOutcomeError> {
    let profile = ctx.profile();
    let discriminator_element = get_element(profile, slicing_descriptor.discriminator)?;
    let all_slice_locs = get_slice_value_locs(&ctx, discriminator_element, value_path);

    let split_slices = split_slicing(ctx.clone(), slicing_descriptor, all_slice_locs).await?;

    let mut issues = Vec::new();
    let elements_pointer = Path::new().descend("snapshot").descend("element");

    for slice in &slicing_descriptor.slices {
        let slice_locs = split_slices.0.get(slice).ok_or_else(|| {
            OperationOutcomeError::error(
                IssueType::exception(),
                format!("Missing slice locations for slice index: {slice}"),
            )
        })?;

        let slice_element_definition = get_element(profile, *slice)?;

        issues.extend(validate_slice_cardinality(
            profile,
            slice_element_definition,
            slice_locs,
        ));

        for slice_loc in slice_locs {
            issues.extend(
                validate_singular_element(
                    ctx.clone(),
                    &elements_pointer.descend(&slice.to_string()),
                    slice_loc,
                )
                .await?,
            );
        }
    }

    Ok(issues)
}
