use haste_fhir_client::url::Parameter;

use super::{
    ClauseTarget, SqlClause, SqlParam, bind, or_predicates, require_values, target_params,
    token_exprs, wrap_predicate,
};
use crate::query::QueryBuildError;

/// Matches `[system|]code`, or with `negate` (`:not`) excludes it. System and
/// code share a row, so no recombination is needed.
pub fn token_clause(
    parsed_parameter: &Parameter,
    target: &ClauseTarget,
    negate: bool,
) -> Result<SqlClause, QueryBuildError> {
    require_values(parsed_parameter)?;

    let (system_column, code_column) = token_exprs(target)?;
    let (predicate, params) = or_predicates(
        &parsed_parameter.value,
        target_params(target),
        |value, params| token_predicate(value, &system_column, &code_column, params),
    )?;

    Ok(wrap_predicate(target, negate, &predicate, params))
}

fn token_predicate(
    value: &str,
    system_column: &str,
    code_column: &str,
    params: Vec<SqlParam>,
) -> Result<(String, Vec<SqlParam>), QueryBuildError> {
    let text = |s: &str| SqlParam::Text(s.to_string());

    Ok(match value.split('|').collect::<Vec<_>>()[..] {
        // `code`: any system.
        [code] => {
            let (params, i) = bind(params, [text(code)]);
            (format!("{code_column} = ${i}"), params)
        }
        // `|`: any token.
        ["", ""] => ("TRUE".to_string(), params),
        // `|code`: no system.
        ["", code] => {
            let (params, i) = bind(params, [text(code)]);
            (
                format!("({code_column} = ${i} AND {system_column} IS NULL)"),
                params,
            )
        }
        // `system|`: any code in the system.
        [system, ""] => {
            let (params, i) = bind(params, [text(system)]);
            (format!("{system_column} = ${i}"), params)
        }
        [system, code] => {
            let (params, i) = bind(params, [text(system), text(code)]);
            (
                format!("({system_column} = ${i} AND {code_column} = ${})", i + 1),
                params,
            )
        }
        _ => return Err(QueryBuildError::InvalidParameterValue(value.to_string())),
    })
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
            false,
        )
        .expect("a token search builds")
        .sql
    }

    /// `_id` has no system; it reads the anchor's primary key column.
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

    /// A repeating token is a correlated `EXISTS` on the shared table.
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
