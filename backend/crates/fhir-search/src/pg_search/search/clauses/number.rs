use haste_fhir_client::url::{Parameter, parse_prefix};

use super::{ClauseTarget, SqlClause, SqlParam, missing_only, require_values};
use crate::{indexing_conversion::get_decimal_range, pg_search::search::QueryBuildError};

pub fn number_clause(
    parsed_parameter: &Parameter,
    target: &ClauseTarget,
) -> Result<SqlClause, QueryBuildError> {
    if missing_only(parsed_parameter)? {
        return target.missing(parsed_parameter);
    }

    require_values(parsed_parameter)?;

    let value_expr = target.value_expr()?;
    let mut params = target.params();
    let predicate = build_or_expr(parsed_parameter, &value_expr, &mut params)?;

    Ok(target.finish(false, &predicate, params))
}

/// OR-joins a predicate per supplied number. A FHIR number carries an implicit
/// precision range, so equality is containment rather than `=`.
fn build_or_expr(
    parsed_parameter: &Parameter,
    value_expr: &str,
    params: &mut Vec<SqlParam>,
) -> Result<String, QueryBuildError> {
    let mut or_clauses = Vec::new();

    for value in &parsed_parameter.value {
        let (prefix, num_str) = parse_prefix(value);
        let range = get_decimal_range(num_str)
            .map_err(|_e| QueryBuildError::InvalidParameterValue(num_str.to_string()))?;

        let low_idx = params.len() + 1;
        let high_idx = params.len() + 2;

        match prefix {
            Some("ne") => {
                or_clauses.push(format!(
                    "NOT ({value_expr} >= ${low_idx} AND {value_expr} <= ${high_idx})"
                ));
                params.push(SqlParam::Float64(range.start));
                params.push(SqlParam::Float64(range.end));
            }
            Some("gt") => {
                or_clauses.push(format!("{value_expr} > ${low_idx}"));
                params.push(SqlParam::Float64(range.end));
            }
            Some("lt") => {
                or_clauses.push(format!("{value_expr} < ${low_idx}"));
                params.push(SqlParam::Float64(range.start));
            }
            Some("ge") => {
                or_clauses.push(format!("{value_expr} >= ${low_idx}"));
                params.push(SqlParam::Float64(range.start));
            }
            Some("le") => {
                or_clauses.push(format!("{value_expr} <= ${low_idx}"));
                params.push(SqlParam::Float64(range.end));
            }
            Some("eq") | None => {
                or_clauses.push(format!(
                    "({value_expr} >= ${low_idx} AND {value_expr} <= ${high_idx})"
                ));
                params.push(SqlParam::Float64(range.start));
                params.push(SqlParam::Float64(range.end));
            }
            Some(p) => return Err(QueryBuildError::UnsupportedPrefix(p.to_string())),
        }
    }

    Ok(or_clauses.join(" OR "))
}
