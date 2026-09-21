use haste_fhir_client::url::Parameter;

use super::{ClauseTarget, SqlClause, SqlParam, direct_column, direct_predicate, dynamic_exists};
use crate::pg_search::{schema::ParamColumns, search::QueryBuildError};

/// The four SQL expressions a quantity predicate reads, in the order the
/// direct-column unnest binds them.
struct QuantityExprs<'a> {
    start: &'a str,
    end: &'a str,
    system: &'a str,
    code: &'a str,
}

pub fn quantity_clause(
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
            let ParamColumns::Quantity {
                start,
                end,
                system,
                code,
            } = columns
            else {
                return Err(QueryBuildError::UnsupportedParameter(
                    "quantity search parameter is not backed by quantity columns".to_string(),
                ));
            };

            // All four sit on the same row, so each value stays beside its
            // own unit with nothing to recombine.
            let (start, end) = (direct_column(alias, start), direct_column(alias, end));
            let (system, code) = (direct_column(alias, system), direct_column(alias, code));

            let mut params = Vec::new();
            let or_expr = build_or_expr(
                parsed_parameter,
                &QuantityExprs {
                    start: &start,
                    end: &end,
                    system: &system,
                    code: &code,
                },
                &mut params,
            )?;

            Ok(SqlClause::new(direct_predicate(false, &or_expr), params))
        }
        ClauseTarget::Dynamic { table, param_url } => {
            // $1 is the param_url discriminator.
            let mut params = vec![SqlParam::Text(param_url.clone())];
            let or_expr = build_or_expr(
                parsed_parameter,
                &QuantityExprs {
                    start: "sq.start_value",
                    end: "sq.end_value",
                    system: "sq.start_system",
                    code: "sq.start_code",
                },
                &mut params,
            )?;

            Ok(SqlClause::new(
                dynamic_exists(table, "sq", false, Some(&or_expr)),
                params,
            ))
        }
    }
}

/// Builds the OR-joined predicate over every supplied `value|system|code`.
/// Empty segments are simply not constrained.
fn build_or_expr(
    parsed_parameter: &Parameter,
    exprs: &QuantityExprs<'_>,
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
