use std::{iter, sync::Arc};

use haste_codegen::{traversal, utilities::extract};
use haste_fhir_client::canonical_resolver::CanonicalResolver;
use haste_fhir_model::r4::{
    generated::{
        resources::{OperationOutcomeIssue, ResourceType},
        terminology::{IssueSeverity, IssueType},
        types::{ElementDefinition, FHIRString},
    },
    get_fhir_type,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_pointer::{Key, Path};

use crate::{
    FHIRProfileCTX,
    slicing::{get_slice_descriptors, validate_slicing_descriptor},
    validators::{cardinality::validate_cardinality, fixed_value, pattern::validate_pattern},
};

fn conformant_to_type(type_to_check: Option<&str>, type_: Option<&str>) -> bool {
    match type_to_check {
        Some("Resource") | Some("DomainResource") => true,
        _ => type_ == type_to_check,
    }
}

/// Check if the element is constrained to profiles type.
/// Also if nested profiles are found, validate against those as well.
///
/// # Arguments
///
/// * `ctx` - The FHIRProfileCTX containing the profile and root data.
/// * `element` - ElementDefinition to check
/// * `type_` - The type found on the element
async fn validate_types_and_profiles_if_present<'a>(
    ctx: Arc<FHIRProfileCTX<'a, impl CanonicalResolver>>,
    element: &ElementDefinition,
    value_pointer: &Path,
    type_: Option<&str>,
) -> Result<Vec<OperationOutcomeIssue>, OperationOutcomeError> {
    if let Some(type_) = type_
        && type_.starts_with("http://hl7.org/fhirpath/System.")
    {
        // FHIRPath system types are not valid types to constrain to, so we ignore them.
        // [TODO] Could consider using the "http://hl7.org/fhir/StructureDefinition/structuredefinition-fhir-type"
        // But this would require an alteration to code generation. Would also have implications on how to validate further the type.
        // As you could end up treating FHIRString for a primitive string type as a complex type with all the extra fields that FHIRString has compared to a primitive string.
        return Ok(vec![]);
    }
    let Some(types) = &element.type_ else {
        return Ok(vec![]);
    };

    if let Some(profile_type) = types
        .iter()
        .find(|t| conformant_to_type(t.code.value.as_ref().map(|s| s.as_str()), type_))
    {
        let mut issues = vec![];

        if let Some(profiles_to_check) = profile_type.profile.as_ref() {
            for profile_canonical in profiles_to_check.iter() {
                if let Some(profile_canonical) = profile_canonical.value.as_ref() {
                    let resolved_resource = ctx
                        .resolver
                        .resolve(ResourceType::StructureDefinition, profile_canonical)
                        .await?
                        .ok_or_else(|| {
                            OperationOutcomeError::error(
                                IssueType::Exception(None),
                                format!(
                                    "Failed to resolve profile canonical: {}",
                                    profile_canonical
                                ),
                            )
                        })?;

                    issues.extend(
                        validate_element(
                            Arc::new(FHIRProfileCTX::new(
                                ctx.resolver.clone(),
                                resolved_resource,
                                ctx.root,
                            )?),
                            &Path::new()
                                .descend("snapshot")
                                .descend("element")
                                .descend("0"),
                            value_pointer,
                        )
                        .await?,
                    );
                }
            }
        }

        Ok(issues)
    } else {
        Ok(vec![outcome_issue(
            &Path::new(),
            IssueSeverity::Error(None),
            IssueType::Value(None),
            format!(
                "Type '{}' is not allowed for element '{}'",
                type_.unwrap_or("unknown"),
                element.id.as_ref().map(|s| s.as_str()).unwrap_or("unknown")
            ),
        )])
    }
}

pub fn outcome_issue(
    value_location: &Path,
    severity: IssueSeverity,
    code: IssueType,
    diagnostic: String,
) -> OperationOutcomeIssue {
    OperationOutcomeIssue {
        severity: Box::new(severity),
        code: Box::new(code),
        diagnostics: Some(Box::new(FHIRString {
            value: Some(diagnostic),
            ..Default::default()
        })),
        location: Some(vec![Box::new(FHIRString {
            value: Some(format!("{}", value_location)),
            ..Default::default()
        })]),
        ..Default::default()
    }
}

pub async fn validate_singular_element<'a>(
    ctx: Arc<FHIRProfileCTX<'a, impl CanonicalResolver>>,
    element_path: &Path,
    value_path: &Path,
) -> Result<Vec<OperationOutcomeIssue>, OperationOutcomeError> {
    let element = element_path
        .get_typed::<Box<ElementDefinition>>(ctx.profile())
        .ok_or_else(|| {
            OperationOutcomeError::error(
                IssueType::Exception(None),
                format!("Invalid elements path: {}", element_path),
            )
        })?;

    let Some(value) = value_path.get(ctx.root) else {
        return Err(OperationOutcomeError::error(
            IssueType::Exception(None),
            format!("Invalid value path: {}", value_path),
        ));
    };
    let mut issues = vec![];
    let Some((elements_pointer, Key::Index(index))) = element_path.ascend() else {
        return Err(OperationOutcomeError::error(
            IssueType::Exception(None),
            format!("Invalid element path: {}", element_path),
        ));
    };

    let elements = elements_pointer
        .get_typed::<Vec<Box<ElementDefinition>>>(ctx.profile())
        .ok_or_else(|| {
            OperationOutcomeError::error(
                IssueType::Exception(None),
                format!("Invalid elements path: {}", elements_pointer),
            )
        })?;

    let children = traversal::ele_index_to_child_indices(elements, index)
        .map_err(|error| OperationOutcomeError::error(IssueType::Exception(None), error))?;

    // Includes all of slice descriptors which is how to split (the descriptor)
    // and the slices that belong to that descriptor (the slices).
    let slice_descriptors = get_slice_descriptors(elements, &children)?;

    let slice_indices_set = slice_descriptors
        .iter()
        .flat_map(|descriptor| {
            descriptor
                .slices
                .iter()
                .chain(iter::once(&descriptor.discriminator))
        })
        .copied()
        .collect::<std::collections::HashSet<usize>>();

    for descriptor in slice_descriptors.iter() {
        issues.extend(
            Box::pin(validate_slicing_descriptor(
                ctx.clone(),
                descriptor,
                value_path,
            ))
            .await?,
        );
    }

    issues.extend(
        validate_types_and_profiles_if_present(
            ctx.clone(),
            element,
            &value_path,
            get_fhir_type(value),
        )
        .await?,
    );

    if let Some(pattern) = element.pattern.as_ref()
        && !validate_pattern(value, pattern)?
    {
        issues.push(outcome_issue(
            value_path,
            IssueSeverity::Error(None),
            IssueType::Value(None),
            format!("Value does not match pattern: {:?}", pattern),
        ));
    }

    if let Some(fixed_value) = element.fixed.as_ref()
        && !fixed_value::is_equal(value, fixed_value)?
    {
        issues.push(outcome_issue(
            value_path,
            IssueSeverity::Error(None),
            IssueType::Value(None),
            format!("Value does not match fixed value: {:?}", fixed_value),
        ));
    }

    // Loop through all children that are not a part of the slice.
    for child in children
        .iter()
        .filter(|child_index| !slice_indices_set.contains(child_index))
    {
        let child_element = &elements[*child];
        let field_name = extract::field_name(
            child_element
                .path
                .value
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or(""),
        );
        let child_element_pointer = elements_pointer.descend(&format!("{}", child));
        let child_value_pointer = value_path.descend(&field_name);
        let child_issues = Box::pin(validate_element(
            ctx.clone(),
            &child_element_pointer,
            &child_value_pointer,
        ))
        .await?;
        issues.extend(child_issues);
    }

    Ok(issues)
}

pub async fn validate_element<'a>(
    ctx: Arc<FHIRProfileCTX<'a, impl CanonicalResolver>>,
    element_pointer: &Path,
    value_pointer: &Path,
) -> Result<Vec<OperationOutcomeIssue>, OperationOutcomeError> {
    let mut issues = vec![];
    let Some(element) = element_pointer.get_typed::<Box<ElementDefinition>>(ctx.profile()) else {
        return Err(OperationOutcomeError::error(
            IssueType::Exception(None),
            format!("Invalid element path: {}", element_pointer),
        ));
    };

    let value = value_pointer.get(ctx.root);

    issues.extend(validate_cardinality(
        ctx.clone(),
        &value_pointer,
        element,
        &value,
    )?);

    if let Some(value) = value {
        if value.is_many() {
            for (i, _v) in value.flatten().iter().enumerate() {
                issues.extend(
                    Box::pin(validate_singular_element(
                        ctx.clone(),
                        element_pointer,
                        &value_pointer.descend(&format!("{}", i)),
                    ))
                    .await?,
                );
            }
        } else {
            issues.extend(
                Box::pin(validate_singular_element(
                    ctx.clone(),
                    element_pointer,
                    value_pointer,
                ))
                .await?,
            );
        }
    }

    Ok(issues)
}
