use crate::{
    elastic_search::search::clauses::any_of,
    query::{QueryBuildError, value::UriValues},
};
use serde_json::json;

/// Exact match on any supplied URI.
pub fn uri(field: &str, values: &UriValues<'_>) -> Result<serde_json::Value, QueryBuildError> {
    let uris = match values {
        UriValues::Uris(uris) => uris,
        // A hierarchy of URIs isn't indexed.
        UriValues::Above(_) | UriValues::Below(_) => {
            return Err(QueryBuildError::unsupported_modifier(values.modifier()));
        }
    };

    Ok(any_of(
        uris.iter()
            .map(|value| json!({ "match": { field: { "query": value } } }))
            .collect(),
    ))
}
