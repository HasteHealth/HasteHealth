use haste_fhir_client::url::{Parameter, parse_prefix};
use haste_fhir_model::r4::datetime::parse_datetime;

use super::{ClauseTarget, SqlClause, SqlParam, direct_column, direct_predicate, dynamic_exists};
use crate::{
    indexing_conversion::date_time_range,
    pg_search::{schema::ParamColumns, search::QueryBuildError},
};

pub fn date_clause(
    parsed_parameter: &Parameter,
    target: &ClauseTarget,
) -> Result<SqlClause, QueryBuildError> {
    if parsed_parameter.value.is_empty() {
        return Err(QueryBuildError::InvalidParameterValue(
            parsed_parameter.name.clone(),
        ));
    }

    match target {
        ClauseTarget::DirectColumn { alias, columns } => {
            let (start_column, end_column) = date_columns(columns)?;

            // One period on the row, so its bounds are read directly and the
            // B-tree on them answers the comparison.
            let mut params = Vec::new();
            let or_expr = build_or_expr(
                parsed_parameter,
                &direct_column(alias, start_column),
                &direct_column(alias, end_column),
                &mut params,
            )?;

            Ok(SqlClause::new(direct_predicate(false, &or_expr), params))
        }
        ClauseTarget::Dynamic { table, param_url } => {
            // $1 is the param_url discriminator.
            let mut params = vec![SqlParam::Text(param_url.clone())];
            let or_expr = build_or_expr(parsed_parameter, "sd.start_ms", "sd.end_ms", &mut params)?;

            Ok(SqlClause::new(
                dynamic_exists(table, "sd", false, Some(&or_expr)),
                params,
            ))
        }
    }
}

fn date_columns(columns: &ParamColumns) -> Result<(&str, &str), QueryBuildError> {
    match columns {
        ParamColumns::Date { start, end } => Ok((start.as_str(), end.as_str())),
        _ => Err(QueryBuildError::UnsupportedParameter(
            "date search parameter is not backed by date columns".to_string(),
        )),
    }
}

/// Builds the OR-joined range predicate over every supplied date value.
///
/// `start_expr`/`end_expr` are the SQL expressions for the *indexed* period's
/// bounds; each search value contributes its own bind parameters.
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
                // Indexed period starts after the search range ends.
                or_clauses.push(format!("{start_expr} > ${first_idx}"));
                params.push(SqlParam::Int64(date_range.end));
            }
            Some("lt") => {
                // Indexed period starts before the search range begins.
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
