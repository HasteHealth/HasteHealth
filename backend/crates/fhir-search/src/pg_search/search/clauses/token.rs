use super::{
    ClauseTarget, SqlClause, SqlParam, all_of, equals, or_predicates, target_params, token_exprs,
    wrap_predicate,
};
use crate::query::{
    QueryBuildError,
    value::{TokenSystem, TokenValue, TokenValues},
};

/// Matches any of the tokens, or with `:not` excludes them. System and code
/// share a row, so no recombination is needed.
pub fn token_clause(
    values: &TokenValues<'_>,
    target: &ClauseTarget,
) -> Result<SqlClause, QueryBuildError> {
    let (tokens, negate) = match values {
        TokenValues::Codes(tokens) => (tokens, false),
        TokenValues::Not(tokens) => (tokens, true),
        // Subsumption and value set membership need the terminology service;
        // text and identifier types aren't indexed.
        TokenValues::Above(_)
        | TokenValues::Below(_)
        | TokenValues::Text(_)
        | TokenValues::In(_)
        | TokenValues::NotIn(_)
        | TokenValues::OfType(_) => {
            return Err(QueryBuildError::unsupported_modifier(values.modifier()));
        }
    };

    let (system_column, code_column) = token_exprs(target)?;
    let (predicate, params) = or_predicates(tokens, target_params(target), |token, params| {
        Ok(token_predicate(token, &system_column, &code_column, params))
    })?;

    Ok(wrap_predicate(target, negate, &predicate, params))
}

/// What the token says of the system and of the code, both required; `|`
/// says nothing of either and matches any token.
fn token_predicate(
    token: &TokenValue<'_>,
    system_column: &str,
    code_column: &str,
    params: Vec<SqlParam>,
) -> (String, Vec<SqlParam>) {
    let (params, system) = match token.system {
        TokenSystem::Any => (params, None),
        TokenSystem::Absent => (params, Some(format!("{system_column} IS NULL"))),
        TokenSystem::Is(system) => equals(params, system_column, Some(system)),
    };
    let (params, code) = equals(params, code_column, token.code);

    (all_of([system, code]), params)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pg_search::{schema::ParamColumns, search::clauses::ANCHOR_TABLE_ALIAS};

    fn search(target: &ClauseTarget, values: &[&str]) -> String {
        let tokens = values
            .iter()
            .map(|value| TokenValue::parse(value).expect("a token parses"))
            .collect::<Vec<_>>();
        token_clause(&TokenValues::Codes(tokens), target)
            .expect("a token search builds")
            .sql
    }

    /// `_id` has no system; it reads the anchor's primary key column.
    #[test]
    fn an_id_search_reads_the_key_column() {
        let target = ClauseTarget::DirectColumn {
            alias: std::borrow::Cow::Borrowed(ANCHOR_TABLE_ALIAS),
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
            correlate: std::borrow::Cow::Borrowed(ANCHOR_TABLE_ALIAS),
        };

        assert_eq!(
            search(&target, &["vital-signs"]),
            "EXISTS (SELECT 1 FROM r4_param_token_idx v \
             WHERE v.res_key = sr.res_key AND v.param_identity = $1 AND (v.code = $2))"
        );
    }
}
