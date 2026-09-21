use haste_fhir_client::url::Parameter;

use super::{ClauseTarget, SqlClause, SqlParam, direct_column, direct_predicate, dynamic_exists};
use crate::pg_search::{schema::ParamColumns, search::QueryBuildError};

pub fn token_clause(
    parsed_parameter: &Parameter,
    target: &ClauseTarget,
) -> Result<SqlClause, QueryBuildError> {
    let negate = match parsed_parameter.modifier.as_deref() {
        Some("not") => true,
        Some(modifier) => {
            return Err(QueryBuildError::UnsupportedModifier(modifier.to_string()));
        }
        None => false,
    };

    if parsed_parameter.value.is_empty() {
        return Err(QueryBuildError::InvalidParameterValue(
            parsed_parameter.name.clone(),
        ));
    }

    match target {
        ClauseTarget::DirectColumn { alias, columns } => {
            let (system_column, code_column) = token_columns(columns)?;

            // Both halves sit on the same row, so the system stays with its
            // own code with nothing to recombine.
            let mut params = Vec::new();
            let or_expr = build_or_expr(
                parsed_parameter,
                &direct_column(alias, system_column),
                &direct_column(alias, code_column),
                &mut params,
            )?;

            Ok(SqlClause::new(direct_predicate(negate, &or_expr), params))
        }
        ClauseTarget::Dynamic { table, param_url } => {
            // $1 is the param_url discriminator.
            let mut params = vec![SqlParam::Text(param_url.clone())];
            let or_expr = build_or_expr(parsed_parameter, "st.system", "st.code", &mut params)?;

            Ok(SqlClause::new(
                dynamic_exists(table, "st", negate, Some(&or_expr)),
                params,
            ))
        }
    }
}

fn token_columns(columns: &ParamColumns) -> Result<(&str, &str), QueryBuildError> {
    match columns {
        ParamColumns::Token { system, code } | ParamColumns::Quantity { system, code, .. } => {
            Ok((system.as_str(), code.as_str()))
        }
        _ => Err(QueryBuildError::UnsupportedParameter(
            "token search parameter is not backed by token columns".to_string(),
        )),
    }
}

/// Builds the OR-joined predicate over every supplied `[system|]code` value,
/// pushing bind parameters onto `params` as it goes.
fn build_or_expr(
    parsed_parameter: &Parameter,
    system_expr: &str,
    code_expr: &str,
    params: &mut Vec<SqlParam>,
) -> Result<String, QueryBuildError> {
    let mut or_clauses = Vec::new();

    for value in &parsed_parameter.value {
        let pieces: Vec<&str> = value.split('|').collect();
        match pieces.len() {
            // code only — match regardless of system
            1 => {
                let idx = params.len() + 1;
                or_clauses.push(format!("{code_expr} = ${idx}"));
                params.push(SqlParam::Text(pieces[0].to_string()));
            }
            2 => {
                let system = pieces[0];
                let code = pieces[1];

                if system.is_empty() && code.is_empty() {
                    // "|" — match any token for this parameter
                    or_clauses.push("TRUE".to_string());
                } else if system.is_empty() {
                    // "|code" — code with no system
                    let idx = params.len() + 1;
                    or_clauses.push(format!("({code_expr} = ${idx} AND {system_expr} IS NULL)"));
                    params.push(SqlParam::Text(code.to_string()));
                } else if code.is_empty() {
                    // "system|" — any code in this system
                    let idx = params.len() + 1;
                    or_clauses.push(format!("{system_expr} = ${idx}"));
                    params.push(SqlParam::Text(system.to_string()));
                } else {
                    let sys_idx = params.len() + 1;
                    let code_idx = params.len() + 2;
                    or_clauses.push(format!(
                        "({system_expr} = ${sys_idx} AND {code_expr} = ${code_idx})"
                    ));
                    params.push(SqlParam::Text(system.to_string()));
                    params.push(SqlParam::Text(code.to_string()));
                }
            }
            _ => return Err(QueryBuildError::InvalidParameterValue(value.clone())),
        }
    }

    Ok(or_clauses.join(" OR "))
}
