use haste_fhir_client::url::{Parameter, parse_prefix};
use haste_fhir_model::r4::datetime::parse_datetime;

use super::{
    ClauseTarget, SqlClause, SqlParam, bind, date_exprs, or_predicates, require_values,
    target_params, wrap_predicate,
};
use crate::{indexing_conversion::date_time_range, pg_search::search::QueryBuildError};

/// Compares each supplied date's range against the indexed period.
pub fn date_clause(
    parsed_parameter: &Parameter,
    target: &ClauseTarget,
) -> Result<SqlClause, QueryBuildError> {
    require_values(parsed_parameter)?;

    let (start, end) = date_exprs(target)?;
    let (predicate, params) = or_predicates(
        &parsed_parameter.value,
        target_params(target),
        |value, params| date_predicate(value, &start, &end, params),
    )?;

    Ok(wrap_predicate(target, false, &predicate, params))
}

fn date_predicate(
    value: &str,
    start: &str,
    end: &str,
    params: Vec<SqlParam>,
) -> Result<(String, Vec<SqlParam>), QueryBuildError> {
    let (prefix, date_str) = parse_prefix(value);
    let invalid = || QueryBuildError::InvalidDateFormat(date_str.to_string());
    let date_time = parse_datetime(date_str).map_err(|_e| invalid())?;
    let range = date_time_range(&date_time).map_err(|_e| invalid())?;
    let (lower, upper) = (SqlParam::Int64(range.start), SqlParam::Int64(range.end));

    Ok(match prefix {
        // Starts after the search range ends.
        Some("gt") => {
            let (params, i) = bind(params, [upper]);
            (format!("{start} > ${i}"), params)
        }
        // Starts before the search range begins.
        Some("lt") => {
            let (params, i) = bind(params, [lower]);
            (format!("{start} < ${i}"), params)
        }
        Some("ge") => {
            let (params, i) = bind(params, [lower]);
            (format!("{start} >= ${i}"), params)
        }
        Some("le") => {
            let (params, i) = bind(params, [upper]);
            (format!("{end} <= ${i}"), params)
        }
        // Doesn't overlap.
        Some("ne") => {
            let (params, i) = bind(params, [lower, upper]);
            (
                format!("NOT ({start} <= ${} AND {end} >= ${i})", i + 1),
                params,
            )
        }
        // Overlaps.
        Some("eq") | None => {
            let (params, i) = bind(params, [lower, upper]);
            (format!("({start} <= ${} AND {end} >= ${i})", i + 1), params)
        }
        Some(p) => return Err(QueryBuildError::UnsupportedPrefix(p.to_string())),
    })
}
