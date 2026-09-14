use haste_fhir_client::url::Parameter;

use super::{ClauseTarget, SqlClause, SqlParam, direct_exists, direct_missing, dynamic_exists};
use crate::pg_search::{schema::ParamColumns, search::QueryBuildError};

pub fn string_clause(
    parsed_parameter: &Parameter,
    target: &ClauseTarget,
) -> Result<SqlClause, QueryBuildError> {
    match parsed_parameter.modifier.as_deref() {
        Some("missing") => missing_clause(target, parsed_parameter),
        Some("exact") => value_clause(target, parsed_parameter, MatchKind::Exact),
        Some("contains") => value_clause(target, parsed_parameter, MatchKind::Contains),
        Some(modifier) => Err(QueryBuildError::UnsupportedModifier(modifier.to_string())),
        None => value_clause(target, parsed_parameter, MatchKind::Prefix),
    }
}

#[derive(Clone, Copy)]
enum MatchKind {
    /// `:exact` — case-sensitive equality.
    Exact,
    /// `:contains` — case-insensitive substring.
    Contains,
    /// Default — case-insensitive prefix.
    Prefix,
}

impl MatchKind {
    /// The comparison for one value, given the SQL expression holding the
    /// indexed string and the placeholder holding the search term.
    fn predicate(self, value_expr: &str, idx: usize) -> String {
        match self {
            MatchKind::Exact => format!("{value_expr} = ${idx}"),
            MatchKind::Contains => {
                format!("LOWER({value_expr}) LIKE LOWER('%' || ${idx} || '%')")
            }
            MatchKind::Prefix => format!("LOWER({value_expr}) LIKE LOWER(${idx} || '%')"),
        }
    }
}

/// The string column for this target. String search parameters always have a
/// single value column.
fn value_column(columns: &ParamColumns) -> Result<&str, QueryBuildError> {
    match columns {
        ParamColumns::String { value } | ParamColumns::Uri { value } => Ok(value.as_str()),
        _ => Err(QueryBuildError::UnsupportedParameter(
            "string search parameter is not backed by a string column".to_string(),
        )),
    }
}

/// `:missing` — whether the parameter produced any value at all.
fn missing_clause(
    target: &ClauseTarget,
    parsed_parameter: &Parameter,
) -> Result<SqlClause, QueryBuildError> {
    let value = parsed_parameter
        .value
        .first()
        .ok_or_else(|| QueryBuildError::InvalidParameterValue(parsed_parameter.name.clone()))?;

    let missing = match value.as_str() {
        "true" => true,
        "false" => false,
        _ => {
            return Err(QueryBuildError::InvalidParameterValue(
                parsed_parameter.name.clone(),
            ));
        }
    };

    match target {
        ClauseTarget::DirectColumn(columns) => Ok(SqlClause::new(
            direct_missing(value_column(columns)?, missing),
            Vec::new(),
        )),
        ClauseTarget::Dynamic { param_url } => Ok(SqlClause::new(
            dynamic_exists("string", "ss", missing, None),
            vec![SqlParam::Text(param_url.clone())],
        )),
    }
}

fn value_clause(
    target: &ClauseTarget,
    parsed_parameter: &Parameter,
    kind: MatchKind,
) -> Result<SqlClause, QueryBuildError> {
    if parsed_parameter.value.is_empty() {
        return Err(QueryBuildError::InvalidParameterValue(
            parsed_parameter.name.clone(),
        ));
    }

    match target {
        ClauseTarget::DirectColumn(columns) => {
            let column = value_column(columns)?;
            let mut params = Vec::new();
            let mut or_clauses = Vec::new();

            for value in &parsed_parameter.value {
                let idx = params.len() + 1;
                or_clauses.push(kind.predicate("v", idx));
                params.push(SqlParam::Text(value.clone()));
            }

            Ok(SqlClause::new(
                direct_exists(&[(column, "v")], false, &or_clauses.join(" OR ")),
                params,
            ))
        }
        ClauseTarget::Dynamic { param_url } => {
            // $1 is the param_url discriminator.
            let mut params = vec![SqlParam::Text(param_url.clone())];
            let mut or_clauses = Vec::new();

            for value in &parsed_parameter.value {
                let idx = params.len() + 1;
                or_clauses.push(kind.predicate("ss.value", idx));
                params.push(SqlParam::Text(value.clone()));
            }

            Ok(SqlClause::new(
                dynamic_exists("string", "ss", false, Some(&or_clauses.join(" OR "))),
                params,
            ))
        }
    }
}
