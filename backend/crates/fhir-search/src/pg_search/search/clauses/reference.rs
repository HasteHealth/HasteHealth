use haste_fhir_client::url::Parameter;

use super::{ClauseTarget, SqlClause, SqlParam, direct_column, direct_predicate, dynamic_exists};
use crate::pg_search::{schema::ParamColumns, search::QueryBuildError};

pub fn reference_clause(
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
            let (type_column, id_column) = reference_columns(columns)?;

            let mut params = Vec::new();
            let or_expr = build_or_expr(
                parsed_parameter,
                &direct_column(alias, type_column),
                &direct_column(alias, id_column),
                &mut params,
            )?;

            Ok(SqlClause::new(direct_predicate(false, &or_expr), params))
        }
        ClauseTarget::Dynamic { table, param_url } => {
            // $1 is the param_url discriminator.
            let mut params = vec![SqlParam::Text(param_url.clone())];
            let or_expr = build_or_expr(
                parsed_parameter,
                "sref.target_resource_type",
                "sref.target_id",
                &mut params,
            )?;

            Ok(SqlClause::new(
                dynamic_exists(table, "sref", false, Some(&or_expr)),
                params,
            ))
        }
    }
}

fn reference_columns(columns: &ParamColumns) -> Result<(&str, &str), QueryBuildError> {
    match columns {
        ParamColumns::Reference {
            target_type,
            target_id,
        } => Ok((target_type.as_str(), target_id.as_str())),
        _ => Err(QueryBuildError::UnsupportedParameter(
            "reference search parameter is not backed by reference columns".to_string(),
        )),
    }
}

/// Builds the OR-joined predicate over every supplied `[Type/]id` value.
fn build_or_expr(
    parsed_parameter: &Parameter,
    type_expr: &str,
    id_expr: &str,
    params: &mut Vec<SqlParam>,
) -> Result<String, QueryBuildError> {
    let mut or_clauses = Vec::new();

    for value in &parsed_parameter.value {
        let pieces: Vec<&str> = value.split('/').collect();
        match pieces.len() {
            // ID only
            1 => {
                let idx = params.len() + 1;
                or_clauses.push(format!("{id_expr} = ${idx}"));
                params.push(SqlParam::Text(pieces[0].to_string()));
            }
            // ResourceType/ID
            2 => {
                let type_idx = params.len() + 1;
                let id_idx = params.len() + 2;
                or_clauses.push(format!(
                    "({type_expr} = ${type_idx} AND {id_expr} = ${id_idx})"
                ));
                params.push(SqlParam::Text(pieces[0].to_string()));
                params.push(SqlParam::Text(pieces[1].to_string()));
            }
            _ => return Err(QueryBuildError::InvalidParameterValue(value.clone())),
        }
    }

    Ok(or_clauses.join(" OR "))
}
