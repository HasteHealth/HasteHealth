use haste_fhir_client::url::{Parameter, parse_prefix};

use super::{
    ClauseTarget, SqlClause, SqlParam, direct_column, direct_missing, direct_predicate,
    dynamic_exists,
};
use crate::{
    indexing_conversion::get_decimal_range,
    pg_search::{schema::ParamColumns, search::QueryBuildError},
};

pub fn number_clause(
    parsed_parameter: &Parameter,
    target: &ClauseTarget,
) -> Result<SqlClause, QueryBuildError> {
    match parsed_parameter.modifier.as_deref() {
        Some("missing") => missing_clause(target, parsed_parameter),
        Some(modifier) => Err(QueryBuildError::UnsupportedModifier(modifier.to_string())),
        None => value_clause(target, parsed_parameter),
    }
}

fn value_column(columns: &ParamColumns) -> Result<&str, QueryBuildError> {
    match columns {
        ParamColumns::Number { value } => Ok(value.as_str()),
        _ => Err(QueryBuildError::UnsupportedParameter(
            "number search parameter is not backed by a number column".to_string(),
        )),
    }
}

fn missing_clause(
    target: &ClauseTarget,
    parsed_parameter: &Parameter,
) -> Result<SqlClause, QueryBuildError> {
    let value = parsed_parameter
        .value
        .first()
        .ok_or_else(|| QueryBuildError::InvalidParameterValue(parsed_parameter.name.clone()))?;

    let missing = match value.as_str() {
        "true" => true,
        "false" => false,
        _ => {
            return Err(QueryBuildError::InvalidParameterValue(
                parsed_parameter.name.clone(),
            ));
        }
    };

    match target {
        ClauseTarget::DirectColumn { alias, columns } => Ok(SqlClause::new(
            direct_missing(alias, value_column(columns)?, missing),
            Vec::new(),
        )),
        ClauseTarget::Dynamic { table, param_url } => Ok(SqlClause::new(
            dynamic_exists(table, "sn", missing, None),
            vec![SqlParam::Text(param_url.clone())],
        )),
    }
}

fn value_clause(
    target: &ClauseTarget,
    parsed_parameter: &Parameter,
) -> Result<SqlClause, QueryBuildError> {
    if parsed_parameter.value.is_empty() {
        return Err(QueryBuildError::InvalidParameterValue(
            parsed_parameter.name.clone(),
        ));
    }

    match target {
        ClauseTarget::DirectColumn { alias, columns } => {
            let column = direct_column(alias, value_column(columns)?);
            let mut params = Vec::new();
            let or_expr = build_or_expr(parsed_parameter, &column, &mut params)?;

            Ok(SqlClause::new(direct_predicate(false, &or_expr), params))
        }
        ClauseTarget::Dynamic { table, param_url } => {
            // $1 is the param_url discriminator.
            let mut params = vec![SqlParam::Text(param_url.clone())];
            let or_expr = build_or_expr(parsed_parameter, "sn.value", &mut params)?;

            Ok(SqlClause::new(
                dynamic_exists(table, "sn", false, Some(&or_expr)),
                params,
            ))
        }
    }
}

/// Builds the OR-joined predicate over every supplied number value. A FHIR
/// number carries an implicit precision range, so equality is a containment
/// test rather than `=`.
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
