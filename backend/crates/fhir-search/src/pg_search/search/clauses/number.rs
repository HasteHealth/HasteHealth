use haste_fhir_client::url::{Parameter, parse_prefix};

use super::{
    ClauseTarget, SqlClause, SqlParam, bind, missing_clause, missing_only, or_predicates,
    require_values, target_params, value_expr, wrap_predicate,
};
use crate::{indexing_conversion::get_decimal_range, pg_search::search::QueryBuildError};

/// A FHIR number has implicit precision, so equality is range containment.
pub fn number_clause(
    parsed_parameter: &Parameter,
    target: &ClauseTarget,
) -> Result<SqlClause, QueryBuildError> {
    if missing_only(parsed_parameter)? {
        return missing_clause(target, parsed_parameter);
    }
    require_values(parsed_parameter)?;

    let column = value_expr(target)?;
    let (predicate, params) = or_predicates(
        &parsed_parameter.value,
        target_params(target),
        |value, params| number_predicate(value, &column, params),
    )?;

    Ok(wrap_predicate(target, false, &predicate, params))
}

fn number_predicate(
    value: &str,
    column: &str,
    params: Vec<SqlParam>,
) -> Result<(String, Vec<SqlParam>), QueryBuildError> {
    let (prefix, num_str) = parse_prefix(value);
    let range = get_decimal_range(num_str)
        .map_err(|_e| QueryBuildError::InvalidParameterValue(num_str.to_string()))?;
    let (low, high) = (SqlParam::Float64(range.start), SqlParam::Float64(range.end));

    Ok(match prefix {
        Some("ne") => {
            let (params, i) = bind(params, [low, high]);
            (
                format!("NOT ({column} >= ${i} AND {column} <= ${})", i + 1),
                params,
            )
        }
        Some("gt") => {
            let (params, i) = bind(params, [high]);
            (format!("{column} > ${i}"), params)
        }
        Some("lt") => {
            let (params, i) = bind(params, [low]);
            (format!("{column} < ${i}"), params)
        }
        Some("ge") => {
            let (params, i) = bind(params, [low]);
            (format!("{column} >= ${i}"), params)
        }
        Some("le") => {
            let (params, i) = bind(params, [high]);
            (format!("{column} <= ${i}"), params)
        }
        Some("eq") | None => {
            let (params, i) = bind(params, [low, high]);
            (
                format!("({column} >= ${i} AND {column} <= ${})", i + 1),
                params,
            )
        }
        Some(p) => return Err(QueryBuildError::UnsupportedPrefix(p.to_string())),
    })
}
