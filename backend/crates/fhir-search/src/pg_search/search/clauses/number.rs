use haste_fhir_client::url::Prefix;

use super::{
    ClauseTarget, SqlClause, SqlParam, bind, or_predicates, target_params, value_expr,
    wrap_predicate,
};
use crate::query::{QueryBuildError, value::NumberValue};

/// A FHIR number has implicit precision, so equality is range containment.
pub fn number_clause(
    values: &[NumberValue],
    target: &ClauseTarget,
) -> Result<SqlClause, QueryBuildError> {
    let column = value_expr(target)?;
    let (predicate, params) = or_predicates(values, target_params(target), |number, params| {
        Ok(number_predicate(number, &column, params))
    })?;

    Ok(wrap_predicate(target, false, &predicate, params))
}

fn number_predicate(
    number: &NumberValue,
    column: &str,
    params: Vec<SqlParam>,
) -> (String, Vec<SqlParam>) {
    let (low, high) = (
        SqlParam::Float64(number.range.start),
        SqlParam::Float64(number.range.end),
    );

    match number.prefix {
        Prefix::Ne => {
            let (params, i) = bind(params, [low, high]);
            (
                format!("NOT ({column} >= ${i} AND {column} <= ${})", i + 1),
                params,
            )
        }
        // For a single value, `sa`/`eb` mean the same as `gt`/`lt`.
        Prefix::Gt | Prefix::Sa => {
            let (params, i) = bind(params, [high]);
            (format!("{column} > ${i}"), params)
        }
        Prefix::Lt | Prefix::Eb => {
            let (params, i) = bind(params, [low]);
            (format!("{column} < ${i}"), params)
        }
        Prefix::Ge => {
            let (params, i) = bind(params, [low]);
            (format!("{column} >= ${i}"), params)
        }
        Prefix::Le => {
            let (params, i) = bind(params, [high]);
            (format!("{column} <= ${i}"), params)
        }
        // `ap` was widened when it was parsed.
        Prefix::Eq | Prefix::Ap => {
            let (params, i) = bind(params, [low, high]);
            (
                format!("({column} >= ${i} AND {column} <= ${})", i + 1),
                params,
            )
        }
    }
}
