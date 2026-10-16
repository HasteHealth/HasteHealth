//! Query-building pieces shared by the search backends: the error type, how a
//! parameter's values and modifier are read ([`value`]), and the page size.

pub mod value;

use crate::{DEFAULT_COUNT, MAX_COUNT, SearchOptions};
use haste_fhir_client::url::Parameter;
use haste_fhir_model::r4::generated::terminology::IssueType;
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
