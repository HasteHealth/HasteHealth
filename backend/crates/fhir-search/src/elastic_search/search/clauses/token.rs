use crate::{
    elastic_search::search::clauses::{all_of, any_of, match_query, nested},
    query::{
        QueryBuildError,
        value::{TokenSystem, TokenValue, TokenValues},
    },
};
use serde_json::json;

/// Matches any of the tokens, and whether `:not` negates the match. The
/// caller applies the negation outside the nested query.
pub fn token(
    field: &str,
    values: &TokenValues<'_>,
) -> Result<(serde_json::Value, bool), QueryBuildError> {
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

    let query = any_of(
        tokens
            .iter()
            .map(|token| nested(field, token_query(field, token)))
            .collect(),
    );
    Ok((query, negate))
}

/// What the token says of the system and of the code, both required; `|`
/// says nothing of either and matches any token.
fn token_query(field: &str, token: &TokenValue<'_>) -> serde_json::Value {
    let mut filter = Vec::new();
    let mut must_not = Vec::new();
    match &token.system {
        TokenSystem::Any => {}
        TokenSystem::Absent => {
            must_not.push(json!({ "exists": { "field": format!("{field}.system") } }));
        }
        TokenSystem::Is(system) => filter.push(match_query(field, "system", system)),
    }
    if let Some(code) = token.code.as_deref() {
        filter.push(match_query(field, "code", code));
    }
    all_of(filter, must_not)
}
