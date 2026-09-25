use haste_fhir_client::url::Parameter;

use super::{
    ClauseTarget, QuantityExprs, SqlClause, SqlParam, bind, or_predicates, quantity_exprs,
    require_values, target_params, wrap_predicate,
};
use crate::pg_search::search::QueryBuildError;

/// Matches `value|system|code`; empty segments are unconstrained. Value and unit
/// share a row, so no recombination is needed.
pub fn quantity_clause(
    parsed_parameter: &Parameter,
    target: &ClauseTarget,
) -> Result<SqlClause, QueryBuildError> {
    require_values(parsed_parameter)?;

    let exprs = quantity_exprs(target)?;
    let (predicate, params) = or_predicates(
        &parsed_parameter.value,
        target_params(target),
        |value, params| quantity_predicate(value, &exprs, params),
    )?;

    Ok(wrap_predicate(target, false, &predicate, params))
}

fn quantity_predicate(
    value: &str,
    exprs: &QuantityExprs,
    params: Vec<SqlParam>,
) -> Result<(String, Vec<SqlParam>), QueryBuildError> {
    let pieces: Vec<&str> = value.split('|').collect();
    let [amount, system, code] = pieces[..] else {
        return Err(if pieces.len() == 4 {
            QueryBuildError::UnsupportedParameterValue(value.to_string())
        } else {
            QueryBuildError::InvalidParameterValue(value.to_string())
        });
    };

    let (params, amount) = if amount.is_empty() {
        (params, None)
    } else {
        let parsed: f64 = amount
            .parse()
            .map_err(|_e| QueryBuildError::InvalidParameterValue(amount.to_string()))?;
        let (params, i) = bind(params, [SqlParam::Float64(parsed)]);
        let sql = format!("({} <= ${i} AND {} >= ${i})", exprs.start, exprs.end);
        (params, Some(sql))
    };
    let (params, system) = equals(params, &exprs.system, system);
    let (params, code) = equals(params, &exprs.code, code);

    let conditions: Vec<String> = [amount, system, code].into_iter().flatten().collect();
    let sql = if conditions.is_empty() {
        "TRUE".to_string()
    } else {
        format!("({})", conditions.join(" AND "))
    };

    Ok((sql, params))
}

/// `column = $n`, or nothing for an empty segment.
fn equals(params: Vec<SqlParam>, column: &str, value: &str) -> (Vec<SqlParam>, Option<String>) {
    if value.is_empty() {
        return (params, None);
    }
    let (params, i) = bind(params, [SqlParam::Text(value.to_string())]);
    (params, Some(format!("{column} = ${i}")))
}
