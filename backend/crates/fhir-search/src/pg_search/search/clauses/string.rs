use super::{
    ClauseTarget, SqlClause, SqlParam, bind, or_predicates, target_params, value_expr,
    wrap_predicate,
};
use crate::query::{QueryBuildError, value::StringValues};

/// Matches any of the strings, compared as their modifier says.
pub fn string_clause(
    values: &StringValues<'_>,
    target: &ClauseTarget,
) -> Result<SqlClause, QueryBuildError> {
    let column = value_expr(target)?;
    let (predicate, params) =
        or_predicates(values.values(), target_params(target), |value, params| {
            let (params, i) = bind(params, [SqlParam::Text(value.clone())]);
            Ok((string_predicate(values, &column, i)?, params))
        })?;

    Ok(wrap_predicate(target, false, &predicate, params))
}

/// Compares `column` against the term bound at `$i`.
fn string_predicate(
    values: &StringValues<'_>,
    column: &str,
    i: usize,
) -> Result<String, QueryBuildError> {
    Ok(match values {
        // Case-insensitive prefix.
        StringValues::Prefix(_) => format!("LOWER({column}) LIKE LOWER(${i} || '%')"),
        // Case-sensitive equality.
        StringValues::Exact(_) => format!("{column} = ${i}"),
        // Case-insensitive substring.
        StringValues::Contains(_) => format!("LOWER({column}) LIKE LOWER('%' || ${i} || '%')"),
        // Needs a text index.
        StringValues::Text(_) => {
            return Err(QueryBuildError::unsupported_modifier(values.modifier()));
        }
    })
}
