use super::{
    ClauseTarget, SqlClause, SqlParam, bind, escape_like, or_predicates, target_params, value_expr,
    wrap_predicate,
};
use crate::query::{
    QueryBuildError,
    value::{UriValues, uri_ancestors},
};

/// Matches any supplied URI: exactly, or by hierarchy for `:above`/`:below`.
pub fn uri_clause(
    values: &UriValues<'_>,
    target: &ClauseTarget,
) -> Result<SqlClause, QueryBuildError> {
    let column = value_expr(target)?;
    let (predicate, params) = match values {
        UriValues::Uris(uris) => or_predicates(uris, target_params(target), |value, params| {
            let (params, i) = bind(params, [SqlParam::Text(value.clone())]);
            Ok((format!("{column} = ${i}"), params))
        })?,
        // Case-sensitive prefix.
        UriValues::Below(uris) => or_predicates(uris, target_params(target), |value, params| {
            let (params, i) = bind(params, [SqlParam::Text(escape_like(value))]);
            Ok((format!("{column} LIKE ${i} || '%' ESCAPE '\\'"), params))
        })?,
        UriValues::Above(uris) => {
            or_predicates(uris, target_params(target), |value, mut params| {
                let first = params.len() + 1;
                let ancestors = uri_ancestors(value);
                let placeholders = (first..first + ancestors.len())
                    .map(|i| format!("${i}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                params.extend(ancestors.into_iter().map(|a| SqlParam::Text(a.to_owned())));
                Ok((format!("{column} IN ({placeholders})"), params))
            })?
        }
    };

    Ok(wrap_predicate(target, false, &predicate, params))
}
