//! Per-type query builders. Each turns a parameter's parsed values into the
//! query on its field; the caller resolves the field and applies `:not` and
//! `:missing` outside it.

use crate::elastic_search::flatten_parameter_field_name;
use haste_fhir_model::r4::generated::resources::SearchParameter;
use serde_json::{Map, Value, json};

mod date;
mod number;
mod quantity;
mod reference;
mod string;
mod token;
mod uri;

pub use date::*;
pub use number::*;
pub use quantity::*;
pub use reference::*;
pub use string::*;
pub use token::*;
pub use uri::*;

/// Resolves the Elasticsearch field path a clause builder should query for
/// this parameter.
///
/// - System-level parameters get their own top-level field, named from the
///   parameter's (fixed, known-at-migration-time) URL - `flatten_parameter_field_name`
///   keeps a literal `.` in that URL from being misread as an object-path
///   separator.
/// - Project-level (user-submitted) parameters instead share one generic
///   `dynamic_parameters` nested field (see `migration.rs`), where the URL is
///   a plain value rather than a field name - an arbitrary user-supplied URL
///   can't safely become a new Elasticsearch field, so it's matched via a
///   `term` query on `dynamic_parameters.url` (added by the caller) instead
///   of being part of this path.
pub fn namespace_parameter(namespace: Option<&str>, search_parameter: &SearchParameter) -> String {
    match namespace {
        None => {
            let url = search_parameter.url.value.as_deref().unwrap_or("");
            flatten_parameter_field_name(url)
        }
        Some(ns) => {
            let type_ = search_parameter.type_.as_str().unwrap_or("string");
            format!("{ns}.value.{type_}")
        }
    }
}

/// Any of `queries`: one per searched value.
fn any_of(queries: Vec<Value>) -> Value {
    object([("bool", object([("should", Value::Array(queries))]))])
}

/// Every one of `filter` and none of `must_not`. A lone filter is bare, and
/// neither is `match_all`.
fn all_of(mut filter: Vec<Value>, must_not: Vec<Value>) -> Value {
    if must_not.is_empty() && filter.len() <= 1 {
        return filter.pop().unwrap_or_else(|| json!({ "match_all": {} }));
    }
    let mut query = Map::new();
    if !filter.is_empty() {
        query.insert("filter".to_string(), Value::Array(filter));
    }
    if !must_not.is_empty() {
        query.insert("must_not".to_string(), Value::Array(must_not));
    }
    object([("bool", Value::Object(query))])
}

/// `query` on the objects nested under `path`.
fn nested(path: &str, query: Value) -> Value {
    object([(
        "nested",
        object([("path", Value::String(path.to_string())), ("query", query)]),
    )])
}

/// `query` matched on `field.name`.
fn match_query(field: &str, name: &str, query: &str) -> Value {
    json!({ "match": { format!("{field}.{name}"): { "query": query } } })
}

/// `field.name` compared to `value` by `comparison` (`gt`, `lte`, ...).
fn range_query(field: &str, name: &str, comparison: &str, value: impl Into<Value>) -> Value {
    json!({ "range": { format!("{field}.{name}"): { comparison: value.into() } } })
}

/// An object of `entries`, moved rather than serialized as `json!` would.
fn object<const N: usize>(entries: [(&str, Value); N]) -> Value {
    Value::Object(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect(),
    )
}
