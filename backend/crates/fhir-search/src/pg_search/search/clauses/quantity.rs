use haste_fhir_client::url::Parameter;

use super::{ClauseTarget, QuantityExprs, SqlClause, SqlParam, require_values};
use crate::pg_search::search::QueryBuildError;

pub fn quantity_clause(
    parsed_parameter: &Parameter,
    target: &ClauseTarget,
) -> Result<SqlClause, QueryBuildError> {
    require_values(parsed_parameter)?;

    // Value and unit sit on the same row, so nothing has to be recombined.
    let exprs = target.quantity_exprs()?;
    let mut params = target.params();
    let predicate = build_or_expr(parsed_parameter, &exprs, &mut params)?;

    Ok(target.finish(false, &predicate, params))
}

/// OR-joins a predicate per supplied `value|system|code`. Empty segments go
/// unconstrained.
fn build_or_expr(
    parsed_parameter: &Parameter,
    exprs: &QuantityExprs,
    params: &mut Vec<SqlParam>,
) -> Result<String, QueryBuildError> {
    let mut or_clauses = Vec::new();

    for value in &parsed_parameter.value {
        let pieces: Vec<&str> = value.split('|').collect();
        match pieces.len() {
            3 => {
                let qty_value = pieces[0];
                let system = pieces[1];
                let code = pieces[2];

                let mut and_clauses = Vec::new();

                if !qty_value.is_empty() {
                    let parsed: f64 = qty_value.parse().map_err(|_e| {
                        QueryBuildError::InvalidParameterValue(qty_value.to_string())
                    })?;
                    let idx = params.len() + 1;
                    and_clauses.push(format!(
                        "({} <= ${idx} AND {} >= ${idx})",
                        exprs.start, exprs.end
                    ));
                    params.push(SqlParam::Float64(parsed));
                }

                if !system.is_empty() {
                    let idx = params.len() + 1;
                    and_clauses.push(format!("{} = ${idx}", exprs.system));
                    params.push(SqlParam::Text(system.to_string()));
                }

                if !code.is_empty() {
                    let idx = params.len() + 1;
                    and_clauses.push(format!("{} = ${idx}", exprs.code));
                    params.push(SqlParam::Text(code.to_string()));
                }

                if and_clauses.is_empty() {
                    or_clauses.push("TRUE".to_string());
                } else {
                    or_clauses.push(format!("({})", and_clauses.join(" AND ")));
                }
            }
            4 => return Err(QueryBuildError::UnsupportedParameterValue(value.clone())),
            _ => return Err(QueryBuildError::InvalidParameterValue(value.clone())),
        }
    }

    Ok(or_clauses.join(" OR "))
}
