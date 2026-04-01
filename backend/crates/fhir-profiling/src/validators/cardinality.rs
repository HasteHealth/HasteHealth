use std::sync::Arc;

use haste_fhir_client::canonical_resolver::CanonicalResolver;
use haste_fhir_model::r4::generated::{
    resources::OperationOutcomeIssue,
    terminology::{IssueSeverity, IssueType},
    types::ElementDefinition,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_pointer::Path;
use haste_reflect::MetaValue;

use crate::{FHIRProfileCTX, element::outcome_issue};

fn _validate_cardinality(
    element: &ElementDefinition,
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
                "Element: '{}' Minimum number of required values not met expected at least '{}', found '{}'",
                element.id.as_ref().map(|s| s.as_str()).unwrap_or("unknown"),
                min,
                value_cardinality
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
                        "Element: '{}' Too many values: expected at most '{}', found '{}'",
                        element.id.as_ref().map(|s| s.as_str()).unwrap_or("unknown"),
                        max,
                        value_cardinality
                    ),
                )])
            }
        } // Missing max defaults to no upper bound at this helper level.
    }
}

pub fn validate_cardinality<'a>(
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
            _validate_cardinality(
                element,
                value_location,
                value_cardinality,
                element_cardinalities,
            )
        }
        None => _validate_cardinality(element, value_location, 0, element_cardinalities),
    }
}
