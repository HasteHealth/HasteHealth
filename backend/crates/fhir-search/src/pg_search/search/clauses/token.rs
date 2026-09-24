use haste_fhir_client::url::Parameter;

use super::{ClauseTarget, SqlClause, SqlParam, require_values};
use crate::pg_search::search::QueryBuildError;

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

    require_values(parsed_parameter)?;

    // Both halves sit on the same row, so nothing has to be recombined.
    let (system_expr, code_expr) = target.token_exprs()?;
    let mut params = target.params();
    let predicate = build_or_expr(parsed_parameter, &system_expr, &code_expr, &mut params)?;

    Ok(target.finish(negate, &predicate, params))
}

/// OR-joins a predicate per supplied `[system|]code`, pushing bind parameters
/// onto `params` as it goes.
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
            // code — any system
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pg_search::{schema::ParamColumns, search::clauses::ANCHOR_TABLE_ALIAS};

    fn search(target: &ClauseTarget, values: &[&str]) -> String {
        token_clause(
            &Parameter {
                name: "code".to_string(),
                modifier: None,
                value: values.iter().map(|v| (*v).to_string()).collect(),
                chains: None,
            },
            target,
        )
        .expect("a token search builds")
        .sql
    }

    /// `_id` has no system: it reads the anchor column the primary key already
    /// indexes.
    #[test]
    fn an_id_search_reads_the_key_column() {
        let target = ClauseTarget::DirectColumn {
            alias: ANCHOR_TABLE_ALIAS,
            columns: ParamColumns::Token {
                system: None,
                code: "resource_id".to_string(),
            },
        };

        assert_eq!(
            search(&target, &["a", "b"]),
            r#"(sr."resource_id" = $1 OR sr."resource_id" = $2)"#
        );
    }

    /// One indexed lookup per value, correlated back to the anchor row: the
    /// planner can drive it either way round.
    #[test]
    fn a_repeating_token_reads_the_shared_table_by_identity() {
        let target = ClauseTarget::Dynamic {
            table: "r4_param_token_idx".to_string(),
            param_identity: 7,
        };

        assert_eq!(
            search(&target, &["vital-signs"]),
            "EXISTS (SELECT 1 FROM r4_param_token_idx v \
             WHERE v.res_key = sr.res_key AND v.param_identity = $1 AND (v.code = $2))"
        );
    }
}
