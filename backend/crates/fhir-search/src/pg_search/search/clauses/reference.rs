use haste_fhir_client::url::Parameter;

use super::{ClauseTarget, SqlClause, SqlParam, require_values};
use crate::pg_search::search::QueryBuildError;

pub fn reference_clause(
    parsed_parameter: &Parameter,
    target: &ClauseTarget,
) -> Result<SqlClause, QueryBuildError> {
    require_values(parsed_parameter)?;

    let (type_expr, id_expr) = target.reference_exprs()?;
    let mut params = target.params();
    let predicate = build_or_expr(parsed_parameter, &type_expr, &id_expr, &mut params)?;

    Ok(target.finish(false, &predicate, params))
}

/// OR-joins a predicate per supplied `[Type/]id` value.
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
