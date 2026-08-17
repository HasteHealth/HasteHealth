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

fn run_validate_cardinality(
    element: &ElementDefinition,
    value_location: &Path,
    value_cardinality: u64,
    (min, max): (u64, Option<&str>),
) -> Result<Vec<OperationOutcomeIssue>, OperationOutcomeError> {
    if value_cardinality < min {
        return Ok(vec![outcome_issue(
            value_location,
            IssueSeverity::error(),
            IssueType::required(),
            format!(
                "Element: '{}' Minimum number of required values not met expected at least '{min}', found '{value_cardinality}'",
                element.id.as_deref().unwrap_or("unknown"),
            ),
        )]);
    }

    match max {
        // "*" means unbounded upper cardinality.
        None | Some("*") => Ok(Vec::new()),
        Some(max) => {
            let Ok(max) = max.parse::<u64>() else {
                return Err(OperationOutcomeError::error(
                    IssueType::exception(),
                    format!("Invalid max cardinality: {max}"),
                ));
            };

            if value_cardinality <= max {
                Ok(Vec::new())
            } else {
                Ok(vec![outcome_issue(
                    value_location,
                    IssueSeverity::error(),
                    IssueType::required(),
                    format!(
                        "Element: '{}' Too many values: expected at most '{max}', found '{value_cardinality}'",
                        element.id.as_deref().unwrap_or("unknown"),
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
    value: Option<&'a dyn MetaValue>,
) -> Result<Vec<OperationOutcomeIssue>, OperationOutcomeError> {
    let element_cardinalities = (
        element.min.as_ref().and_then(|v| v.value).unwrap_or(0),
        element.max.as_ref().and_then(|v| v.value.as_deref()),
    );

    match value {
        Some(v) => {
            let value_cardinality = v.flatten().len() as u64;
            run_validate_cardinality(
                element,
                value_location,
                value_cardinality,
                element_cardinalities,
            )
        }
        None => run_validate_cardinality(element, value_location, 0, element_cardinalities),
    }
}
