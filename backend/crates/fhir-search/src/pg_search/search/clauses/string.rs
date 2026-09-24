use haste_fhir_client::url::Parameter;

use super::{ClauseTarget, SqlClause, SqlParam, require_values};
use crate::pg_search::search::QueryBuildError;

pub fn string_clause(
    parsed_parameter: &Parameter,
    target: &ClauseTarget,
) -> Result<SqlClause, QueryBuildError> {
    let kind = match parsed_parameter.modifier.as_deref() {
        Some("missing") => return target.missing(parsed_parameter),
        Some("exact") => MatchKind::Exact,
        Some("contains") => MatchKind::Contains,
        Some(modifier) => {
            return Err(QueryBuildError::UnsupportedModifier(modifier.to_string()));
        }
        None => MatchKind::Prefix,
    };

    require_values(parsed_parameter)?;

    let value_expr = target.value_expr()?;
    let mut params = target.params();
    let mut or_clauses = Vec::new();

    for value in &parsed_parameter.value {
        let idx = params.len() + 1;
        or_clauses.push(kind.predicate(&value_expr, idx));
        params.push(SqlParam::Text(value.clone()));
    }

    Ok(target.finish(false, &or_clauses.join(" OR "), params))
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
    /// Compares the indexed string in `value_expr` against the term bound at
    /// `idx`.
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
