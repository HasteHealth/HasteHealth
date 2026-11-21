use crate::{
    elastic_search::search::clauses::any_of,
    query::value::{UriValues, uri_ancestors},
};
use serde_json::json;

/// Matches any supplied URI: exactly, or by hierarchy for `:above`/`:below`.
pub fn uri(field: &str, values: &UriValues<'_>) -> serde_json::Value {
    any_of(match values {
        UriValues::Uris(uris) => uris
            .iter()
            .map(|value| json!({ "match": { field: { "query": value } } }))
            .collect(),
        // A `prefix` query on a keyword field is literal; nothing to escape.
        UriValues::Below(uris) => uris
            .iter()
            .map(|value| json!({ "prefix": { field: { "value": value } } }))
            .collect(),
        UriValues::Above(uris) => uris
            .iter()
            .map(|value| json!({ "terms": { field: uri_ancestors(value) } }))
            .collect(),
    })
}
