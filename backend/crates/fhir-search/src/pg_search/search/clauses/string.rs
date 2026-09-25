use haste_fhir_client::url::Parameter;

use super::{
    ClauseTarget, SqlClause, SqlParam, bind, or_predicates, require_values, target_params,
    value_expr, wrap_predicate,
};
use crate::query::{Modifier, QueryBuildError};

/// `modifier` is `Exact`, `Contains` or `None` (already validated).
pub fn string_clause(
    parsed_parameter: &Parameter,
    target: &ClauseTarget,
    modifier: Modifier,
) -> Result<SqlClause, QueryBuildError> {
    require_values(parsed_parameter)?;

    let column = value_expr(target)?;
    let (predicate, params) = or_predicates(
        &parsed_parameter.value,
        target_params(target),
        |value, params| {
            let (params, i) = bind(params, [SqlParam::Text(value.to_string())]);
            Ok((string_predicate(modifier, &column, i), params))
        },
    )?;

    Ok(wrap_predicate(target, false, &predicate, params))
}

/// Compares `column` against the term bound at `$i`.
fn string_predicate(modifier: Modifier, column: &str, i: usize) -> String {
    match modifier {
        // Case-sensitive equality.
        Modifier::Exact => format!("{column} = ${i}"),
        // Case-insensitive substring.
        Modifier::Contains => format!("LOWER({column}) LIKE LOWER('%' || ${i} || '%')"),
        // Default: case-insensitive prefix.
        _ => format!("LOWER({column}) LIKE LOWER(${i} || '%')"),
    }
}
