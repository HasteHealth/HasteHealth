//! Query-building pieces shared by the Elasticsearch and Postgres backends:
//! the error type and which modifiers each parameter type accepts.

use haste_fhir_client::url::Parameter;
use haste_fhir_model::r4::generated::terminology::{BoundCode, SearchParamType};
use haste_fhir_operation_error::derive::OperationOutcomeError;

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
        code = "not-supported",
        diagnostic = "Prefix '{arg0}' is not supported for this search type."
    )]
    UnsupportedPrefix(String),
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

/// A search parameter's modifier, validated against its type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modifier {
    None,
    /// `:missing=true` (`true`) or `:missing=false` (`false`). Any stored type.
    Missing(bool),
    /// `:not`. Token only.
    Not,
    /// `:exact`. String only.
    Exact,
    /// `:contains`. String only.
    Contains,
}

/// Parses `parameter`'s modifier and checks that `param_type` supports it.
///
/// # Errors
///
/// Returns [`QueryBuildError::UnsupportedModifier`] for a modifier the type
/// doesn't support, or [`QueryBuildError::InvalidParameterValue`] unless a
/// `:missing` value is exactly `true` or `false`.
pub fn parse_modifier(
    parameter: &Parameter,
    param_type: &BoundCode<SearchParamType>,
) -> Result<Modifier, QueryBuildError> {
    let is = |candidate: BoundCode<SearchParamType>| param_type == &candidate;

    match parameter.modifier.as_deref() {
        None => Ok(Modifier::None),
        Some("missing") if is_stored(param_type) => match parameter.value.as_slice() {
            [value] if value == "true" => Ok(Modifier::Missing(true)),
            [value] if value == "false" => Ok(Modifier::Missing(false)),
            _ => Err(QueryBuildError::InvalidParameterValue(
                parameter.name.clone(),
            )),
        },
        Some("not") if is(SearchParamType::token()) => Ok(Modifier::Not),
        Some("exact") if is(SearchParamType::string()) => Ok(Modifier::Exact),
        Some("contains") if is(SearchParamType::string()) => Ok(Modifier::Contains),
        Some(modifier) => Err(QueryBuildError::UnsupportedModifier(modifier.to_string())),
    }
}

/// Types both backends index (everything except composite and special).
fn is_stored(param_type: &BoundCode<SearchParamType>) -> bool {
    [
        SearchParamType::string(),
        SearchParamType::token(),
        SearchParamType::date(),
        SearchParamType::number(),
        SearchParamType::quantity(),
        SearchParamType::uri(),
        SearchParamType::reference(),
    ]
    .contains(param_type)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(
        type_: &BoundCode<SearchParamType>,
        modifier: &str,
        values: &[&str],
    ) -> Result<Modifier, QueryBuildError> {
        parse_modifier(
            &Parameter {
                name: "p".to_string(),
                modifier: Some(modifier.to_string()),
                value: values.iter().map(|v| (*v).to_string()).collect(),
                chains: None,
            },
            type_,
        )
    }

    #[test]
    fn missing_is_accepted_for_every_stored_type() {
        assert_eq!(
            parse(&SearchParamType::reference(), "missing", &["true"]).unwrap(),
            Modifier::Missing(true)
        );
        assert_eq!(
            parse(&SearchParamType::date(), "missing", &["false"]).unwrap(),
            Modifier::Missing(false)
        );
        assert!(parse(&SearchParamType::composite(), "missing", &["true"]).is_err());
        assert!(parse(&SearchParamType::date(), "missing", &["maybe"]).is_err());
        assert!(parse(&SearchParamType::date(), "missing", &["true", "false"]).is_err());
    }

    #[test]
    fn type_specific_modifiers_are_limited_to_their_type() {
        assert_eq!(
            parse(&SearchParamType::token(), "not", &["a"]).unwrap(),
            Modifier::Not
        );
        assert_eq!(
            parse(&SearchParamType::string(), "exact", &["a"]).unwrap(),
            Modifier::Exact
        );
        assert!(parse(&SearchParamType::string(), "not", &["a"]).is_err());
        assert!(parse(&SearchParamType::token(), "exact", &["a"]).is_err());
        assert!(parse(&SearchParamType::reference(), "foo", &["a"]).is_err());
    }
}
