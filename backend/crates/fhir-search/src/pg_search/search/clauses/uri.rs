use super::{
    ClauseTarget, SqlClause, SqlParam, bind, or_predicates, target_params, value_expr,
    wrap_predicate,
};
use crate::query::{QueryBuildError, value::UriValues};

/// Exact match on any supplied URI.
pub fn uri_clause(
    values: &UriValues<'_>,
    target: &ClauseTarget,
) -> Result<SqlClause, QueryBuildError> {
    let uris = match values {
        UriValues::Uris(uris) => uris,
        // A hierarchy of URIs isn't indexed.
        UriValues::Above(_) | UriValues::Below(_) => {
            return Err(QueryBuildError::unsupported_modifier(values.modifier()));
        }
    };

    let column = value_expr(target)?;
    let (predicate, params) = or_predicates(uris, target_params(target), |value, params| {
        let (params, i) = bind(params, [SqlParam::Text(value.clone())]);
        Ok((format!("{column} = ${i}"), params))
    })?;

    Ok(wrap_predicate(target, false, &predicate, params))
}
