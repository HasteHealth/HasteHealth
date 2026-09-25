use haste_fhir_client::url::{Parameter, parse_prefix};

use super::{
    ClauseTarget, QuantityExprs, SqlClause, SqlParam, bind, or_predicates, quantity_exprs,
    require_values, target_params, wrap_predicate,
};
use crate::{
    indexing_conversion::get_decimal_range,
    query::QueryBuildError,
    search_ranges::{above, approximate_decimal, below},
};

/// Matches `[prefix]value[|system|code]`; empty segments are unconstrained.
/// Value and unit share a row, so no recombination is needed.
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
    let (amount, system, code) = match value.split('|').collect::<Vec<_>>()[..] {
        // A bare value matches any unit.
        [amount] => (amount, "", ""),
        [amount, system, code] => (amount, system, code),
        [_, _, _, _] => {
            return Err(QueryBuildError::UnsupportedParameterValue(
                value.to_string(),
            ));
        }
        _ => return Err(QueryBuildError::InvalidParameterValue(value.to_string())),
    };

    let (params, amount) = if amount.is_empty() {
        (params, None)
    } else {
        let (params, sql) = amount_predicate(amount, exprs, params)?;
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

/// Compares the indexed `[start, end]` range against `[prefix]amount`.
fn amount_predicate(
    amount: &str,
    exprs: &QuantityExprs,
    params: Vec<SqlParam>,
) -> Result<(Vec<SqlParam>, String), QueryBuildError> {
    let (prefix, number) = parse_prefix(amount);
    let invalid = || QueryBuildError::InvalidParameterValue(amount.to_string());
    let range = get_decimal_range(number).map_err(|_e| invalid())?;
    let (start, end) = (&exprs.start, &exprs.end);
    let float = SqlParam::Float64;

    Ok(match prefix {
        // The value falls inside the indexed range.
        Some("eq" | "ne") | None => {
            let point: f64 = number.parse().map_err(|_e| invalid())?;
            let (params, i) = bind(params, [float(point)]);
            let contains = format!("({start} <= ${i} AND {end} >= ${i})");
            let sql = if prefix == Some("ne") {
                format!("NOT {contains}")
            } else {
                contains
            };
            (params, sql)
        }
        // Wholly above or below the search value's precision range.
        Some("gt" | "sa") => {
            let (params, i) = bind(params, [float(above(&range))]);
            (params, format!("{start} >= ${i}"))
        }
        Some("lt" | "eb") => {
            let (params, i) = bind(params, [float(below(&range))]);
            (params, format!("{end} <= ${i}"))
        }
        Some("ge") => {
            let (params, i) = bind(params, [float(range.start)]);
            (params, format!("{start} >= ${i}"))
        }
        Some("le") => {
            let (params, i) = bind(params, [float(range.end)]);
            (params, format!("{end} <= ${i}"))
        }
        // Overlaps the value widened by 10%.
        Some("ap") => {
            let (low, high) = approximate_decimal(&range);
            let (params, i) = bind(params, [float(low), float(high)]);
            (params, format!("({start} <= ${} AND {end} >= ${i})", i + 1))
        }
        Some(p) => return Err(QueryBuildError::UnsupportedPrefix(p.to_string())),
    })
}

/// `column = $n`, or nothing for an empty segment.
fn equals(params: Vec<SqlParam>, column: &str, value: &str) -> (Vec<SqlParam>, Option<String>) {
    if value.is_empty() {
        return (params, None);
    }
    let (params, i) = bind(params, [SqlParam::Text(value.to_string())]);
    (params, Some(format!("{column} = ${i}")))
}
