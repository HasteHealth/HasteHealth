//! Query-building pieces shared by the search backends: the error type, how a
//! parameter's values and modifier are read ([`value`]), and the page size.

pub mod value;

use crate::{DEFAULT_COUNT, MAX_COUNT, SearchOptions};
use haste_fhir_client::{
    request::SearchRequest,
    url::{Parameter, ParsedParameter},
};
use haste_fhir_model::r4::generated::{resources::ResourceType, terminology::IssueType};
use haste_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};

#[derive(OperationOutcomeError, Debug)]
pub enum QueryBuildError {
    #[error(
        code = "not-found",
        diagnostic = "Search parameter with name '{arg0}' not found.'"
    )]
    MissingParameter(String),
    #[error(code = "not-supported", diagnostic = "Unsupported parameter: '{arg0}'")]
    UnsupportedParameter(String),
    #[error(
        code = "not-supported",
        diagnostic = "Unsupported sorting parameter: '{arg0}'"
    )]
    UnsupportedSortParameter(String),
    #[error(
        code = "not-supported",
        diagnostic = "Unsupported modifier parameter: '{arg0}'"
    )]
    UnsupportedModifier(String),
    #[error(
        code = "invalid",
        diagnostic = "Modifier '{arg0}' does not apply to this search parameter type."
    )]
    InvalidModifier(String),
    #[error(
        code = "not-supported",
        diagnostic = "Parameter value '{arg0}' is not supported for this search type."
    )]
    UnsupportedParameterValue(String),
    #[error(code = "invalid", diagnostic = "Invalid parameter value: '{arg0}'")]
    InvalidParameterValue(String),
    #[error(code = "invalid", diagnostic = "Invalid date format: '{arg0}'")]
    InvalidDateFormat(String),
}

impl QueryBuildError {
    /// A modifier the specification defines that this backend doesn't search.
    #[must_use]
    pub fn unsupported_modifier(code: &str) -> Self {
        QueryBuildError::UnsupportedModifier(code.to_string())
    }
}

/// The page size `options` allows: its limit, or the default.
///
/// # Errors
///
/// Returns an error for a limit above [`MAX_COUNT`].
pub(crate) fn max_count(options: Option<&SearchOptions>) -> Result<u64, OperationOutcomeError> {
    match options.and_then(|options| options.count_limit) {
        Some(limit) if limit > MAX_COUNT => Err(OperationOutcomeError::fatal(
            IssueType::too_costly(),
            "Count limit exceeds maximum allowed.".to_string(),
        )),
        Some(limit) => Ok(limit),
        None => Ok(DEFAULT_COUNT),
    }
}

/// A result parameter's single numeric value (`_count`, `_offset`).
///
/// # Errors
///
/// Returns an error when the value is missing or not a number.
pub(crate) fn parse_u64(parameter: &Parameter) -> Result<u64, OperationOutcomeError> {
    let value = parameter.value.first().ok_or_else(|| {
        OperationOutcomeError::error(
            IssueType::required(),
            format!("Missing parameter value: {}", parameter.name),
        )
    })?;
    value.parse().map_err(|_| {
        OperationOutcomeError::fatal(
            IssueType::invalid(),
            format!("Invalid {} value: '{value}'.", parameter.name),
        )
    })
}

/// The types a system-level search's `_type` restricts it to. `None` when the
/// search is at the type level or names no `_type`.
///
/// # Errors
///
/// Returns an error for a value that is not a resource type.
pub(crate) fn type_filter(
    request: &SearchRequest,
) -> Result<Option<Vec<ResourceType>>, QueryBuildError> {
    let SearchRequest::System(request) = request else {
        return Ok(None);
    };
    let Some(parameter) = request.parameters.get("_type") else {
        return Ok(None);
    };
    let (ParsedParameter::Result(parameter) | ParsedParameter::Resource(parameter)) = parameter;

    parameter
        .value
        .iter()
        .filter(|value| !value.is_empty())
        .map(|value| {
            ResourceType::try_from(value.as_str())
                .map_err(|_| QueryBuildError::InvalidParameterValue(format!("_type={value}")))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;
    use haste_fhir_client::{
        request::{FHIRSearchSystemRequest, FHIRSearchTypeRequest},
        url::ParsedParameters,
    };

    fn system(query: &str) -> SearchRequest {
        SearchRequest::System(FHIRSearchSystemRequest {
            parameters: ParsedParameters::try_from(query).expect("query parses"),
        })
    }

    #[test]
    fn type_filter_reads_the_named_types() {
        assert_eq!(
            type_filter(&system("_type=Patient,Observation")).expect("valid"),
            Some(vec![ResourceType::Patient, ResourceType::Observation])
        );
        assert_eq!(type_filter(&system("_tag=x")).expect("valid"), None);
    }

    #[test]
    fn type_filter_refuses_an_unknown_type() {
        assert!(type_filter(&system("_type=Patient,NotAType")).is_err());
    }

    #[test]
    fn type_filter_ignores_a_type_level_search() {
        let request = SearchRequest::Type(FHIRSearchTypeRequest {
            resource_type: ResourceType::Patient,
            parameters: ParsedParameters::try_from("_type=Observation").expect("query parses"),
        });
        assert_eq!(type_filter(&request).expect("valid"), None);
    }
}
