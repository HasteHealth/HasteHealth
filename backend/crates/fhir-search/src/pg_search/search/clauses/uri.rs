use haste_fhir_client::url::Parameter;

use super::{
    ClauseTarget, SqlClause, SqlParam, bind, or_predicates, require_values, target_params,
    value_expr, wrap_predicate,
};
use crate::query::QueryBuildError;

/// Exact match on any supplied URI.
pub fn uri_clause(
    parsed_parameter: &Parameter,
    target: &ClauseTarget,
) -> Result<SqlClause, QueryBuildError> {
    require_values(parsed_parameter)?;

    let column = value_expr(target)?;
    let (predicate, params) = or_predicates(
        &parsed_parameter.value,
        target_params(target),
        |value, params| {
            let (params, i) = bind(params, [SqlParam::Text(value.to_string())]);
            Ok((format!("{column} = ${i}"), params))
        },
    )?;

    Ok(wrap_predicate(target, false, &predicate, params))
}
