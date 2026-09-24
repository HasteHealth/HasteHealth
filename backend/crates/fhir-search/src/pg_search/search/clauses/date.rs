use haste_fhir_client::url::{Parameter, parse_prefix};
use haste_fhir_model::r4::datetime::parse_datetime;

use super::{ClauseTarget, SqlClause, SqlParam, require_values};
use crate::{indexing_conversion::date_time_range, pg_search::search::QueryBuildError};

pub fn date_clause(
    parsed_parameter: &Parameter,
    target: &ClauseTarget,
) -> Result<SqlClause, QueryBuildError> {
    require_values(parsed_parameter)?;

    // One period per row: its bounds are read directly and the B-tree on them
    // answers the comparison.
    let (start_expr, end_expr) = target.date_exprs()?;
    let mut params = target.params();
    let predicate = build_or_expr(parsed_parameter, &start_expr, &end_expr, &mut params)?;

    Ok(target.finish(false, &predicate, params))
}

/// OR-joins a range predicate per supplied date, against the indexed period's
/// bounds.
fn build_or_expr(
    parsed_parameter: &Parameter,
    start_expr: &str,
    end_expr: &str,
    params: &mut Vec<SqlParam>,
) -> Result<String, QueryBuildError> {
    let mut or_clauses = Vec::new();

    for value in &parsed_parameter.value {
        let (prefix, date_str) = parse_prefix(value);

        let date_time = parse_datetime(date_str)
            .map_err(|_e| QueryBuildError::InvalidDateFormat(date_str.to_string()))?;

        let date_range = date_time_range(&date_time)
            .map_err(|_e| QueryBuildError::InvalidDateFormat(date_str.to_string()))?;

        let first_idx = params.len() + 1;
        let second_idx = params.len() + 2;

        match prefix {
            Some("gt") => {
                // Starts after the search range ends.
                or_clauses.push(format!("{start_expr} > ${first_idx}"));
                params.push(SqlParam::Int64(date_range.end));
            }
            Some("lt") => {
                // Starts before the search range begins.
                or_clauses.push(format!("{start_expr} < ${first_idx}"));
                params.push(SqlParam::Int64(date_range.start));
            }
            Some("ge") => {
                or_clauses.push(format!("{start_expr} >= ${first_idx}"));
                params.push(SqlParam::Int64(date_range.start));
            }
            Some("le") => {
                or_clauses.push(format!("{end_expr} <= ${first_idx}"));
                params.push(SqlParam::Int64(date_range.end));
            }
            Some("ne") => {
                // Not overlapping.
                or_clauses.push(format!(
                    "NOT ({start_expr} <= ${second_idx} AND {end_expr} >= ${first_idx})"
                ));
                params.push(SqlParam::Int64(date_range.start));
                params.push(SqlParam::Int64(date_range.end));
            }
            Some("eq") | None => {
                // Overlapping.
                or_clauses.push(format!(
                    "({start_expr} <= ${second_idx} AND {end_expr} >= ${first_idx})"
                ));
                params.push(SqlParam::Int64(date_range.start));
                params.push(SqlParam::Int64(date_range.end));
            }
            Some(p) => return Err(QueryBuildError::UnsupportedPrefix(p.to_string())),
        }
    }

    Ok(or_clauses.join(" OR "))
}
