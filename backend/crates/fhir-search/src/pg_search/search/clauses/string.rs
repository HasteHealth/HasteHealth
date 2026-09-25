use haste_fhir_client::url::Parameter;

use super::{
    ClauseTarget, SqlClause, SqlParam, bind, missing_clause, or_predicates, require_values,
    target_params, value_expr, wrap_predicate,
};
use crate::pg_search::search::QueryBuildError;

#[derive(Clone, Copy)]
enum MatchKind {
    /// `:exact`: case-sensitive equality.
    Exact,
    /// `:contains`: case-insensitive substring.
    Contains,
    /// Default: case-insensitive prefix.
    Prefix,
}

pub fn string_clause(
    parsed_parameter: &Parameter,
    target: &ClauseTarget,
) -> Result<SqlClause, QueryBuildError> {
    let kind = match parsed_parameter.modifier.as_deref() {
        Some("missing") => return missing_clause(target, parsed_parameter),
        Some("exact") => MatchKind::Exact,
        Some("contains") => MatchKind::Contains,
        Some(modifier) => {
            return Err(QueryBuildError::UnsupportedModifier(modifier.to_string()));
        }
        None => MatchKind::Prefix,
    };
    require_values(parsed_parameter)?;

    let column = value_expr(target)?;
    let (predicate, params) = or_predicates(
        &parsed_parameter.value,
        target_params(target),
        |value, params| {
            let (params, i) = bind(params, [SqlParam::Text(value.to_string())]);
            Ok((string_predicate(kind, &column, i), params))
        },
    )?;

    Ok(wrap_predicate(target, false, &predicate, params))
}

/// Compares `column` against the term bound at `$i`.
fn string_predicate(kind: MatchKind, column: &str, i: usize) -> String {
    match kind {
        MatchKind::Exact => format!("{column} = ${i}"),
        MatchKind::Contains => format!("LOWER({column}) LIKE LOWER('%' || ${i} || '%')"),
        MatchKind::Prefix => format!("LOWER({column}) LIKE LOWER(${i} || '%')"),
    }
}
