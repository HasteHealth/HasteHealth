use std::sync::Arc;

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
use haste_reflect::MetaValue;

use crate::FHIRProfileCTX;

/// Check if the element is constrained to profiles type.
/// Also if nested profiles are found, validate against those as well.
///
/// # Arguments
///
/// * `ctx` - The FHIRProfileCTX containing the profile and root data.
/// * `element` - ElementDefinition to check
/// * `type_` - The type found on the element
async fn validate_type_if_multiple_types_constrained<'a>(
    ctx: Arc<FHIRProfileCTX<'a, impl CanonicalResolver>>,
    element: &ElementDefinition,
    value_pointer: &Path,
    type_: Option<&str>,
) -> Result<Vec<OperationOutcomeIssue>, OperationOutcomeError> {
    let Some(types) = &element.type_ else {
        return Ok(vec![]);
    };

    if let Some(profile_type) = types
        .iter()
        .find(|t| t.code.value.as_ref().map(|s| s.as_str()) == type_)
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
            IssueType::Required(None),
            format!(
                "Type '{}' is not allowed for this element",
                type_.unwrap_or("unknown")
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

fn _validate_cardinality(
    value_location: &Path,
    value_cardinality: usize,
    (min, max): (usize, Option<&str>),
) -> Result<Vec<OperationOutcomeIssue>, OperationOutcomeError> {
    if value_cardinality < min {
        return Ok(vec![outcome_issue(
            value_location,
            IssueSeverity::Error(None),
            IssueType::Required(None),
            format!(
                "Cardinality too low: expected at least '{}', found '{}'",
                min, value_cardinality
            ),
        )]);
    }

    match max {
        // "*" means unbounded upper cardinality.
        None | Some("*") => Ok(vec![]),
        Some(max) => {
            let Ok(max) = max.parse::<usize>() else {
                return Err(OperationOutcomeError::error(
                    IssueType::Exception(None),
                    format!("Invalid max cardinality: {}", max),
                ));
            };

            if value_cardinality <= max {
                Ok(vec![])
            } else {
                Ok(vec![outcome_issue(
                    value_location,
                    IssueSeverity::Error(None),
                    IssueType::Required(None),
                    format!(
                        "Cardinality too high: expected at most '{}', found '{}'",
                        max, value_cardinality
                    ),
                )])
            }
        } // Missing max defaults to no upper bound at this helper level.
    }
}

fn validate_cardinality<'a>(
    _ctx: Arc<FHIRProfileCTX<'a, impl CanonicalResolver>>,
    value_location: &Path,
    element: &ElementDefinition,
    value: &Option<&'a dyn MetaValue>,
) -> Result<Vec<OperationOutcomeIssue>, OperationOutcomeError> {
    let element_cardinalities = (
        element.min.as_ref().and_then(|v| v.value).unwrap_or(0) as usize,
        element
            .max
            .as_ref()
            .and_then(|v| v.value.as_ref().map(|s| s.as_str())),
    );

    match value {
        Some(v) => {
            let value_cardinality = v.flatten().len();
            _validate_cardinality(value_location, value_cardinality, element_cardinalities)
        }
        None => _validate_cardinality(value_location, 0, element_cardinalities),
    }
}

async fn validate_singular_element<'a>(
    ctx: Arc<FHIRProfileCTX<'a, impl CanonicalResolver>>,
    element_pointer: &Path,
    value_pointer: &Path,
    element: &ElementDefinition,
    value: &'a dyn MetaValue,
) -> Result<Vec<OperationOutcomeIssue>, OperationOutcomeError> {
    let mut issues = vec![];
    let Some((elements_pointer, Key::Index(index))) = element_pointer.ascend() else {
        return Err(OperationOutcomeError::error(
            IssueType::Exception(None),
            format!("Invalid element path: {}", element_pointer),
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

    issues.extend(
        validate_type_if_multiple_types_constrained(
            ctx.clone(),
            element,
            &value_pointer,
            get_fhir_type(value),
        )
        .await?,
    );

    let children = traversal::ele_index_to_child_indices(elements, index)
        .map_err(|error| OperationOutcomeError::error(IssueType::Exception(None), error))?;

    for child in children.iter() {
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
        let child_value_pointer = value_pointer.descend(&field_name);
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
            for (i, v) in value.flatten().iter().enumerate() {
                issues.extend(
                    validate_singular_element(
                        ctx.clone(),
                        element_pointer,
                        &value_pointer.descend(&format!("{}", i)),
                        element,
                        *v,
                    )
                    .await?,
                );
            }
        } else {
            issues.extend(
                validate_singular_element(
                    ctx.clone(),
                    element_pointer,
                    value_pointer,
                    element,
                    value,
                )
                .await?,
            );
        }
    }

    Ok(issues)
}
