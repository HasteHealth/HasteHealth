use crate::{
    elastic_search::search::clauses::any_of,
    query::{QueryBuildError, value::StringValues},
};
use serde_json::json;

/// Matches any of the strings, compared as their modifier says.
pub fn string(
    field: &str,
    values: &StringValues<'_>,
) -> Result<serde_json::Value, QueryBuildError> {
    let queries = values
        .values()
        .iter()
        .map(|value| string_query(field, values, value))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(any_of(queries))
}

fn string_query(
    field: &str,
    values: &StringValues<'_>,
    value: &str,
) -> Result<serde_json::Value, QueryBuildError> {
    Ok(match values {
        // Case-insensitive prefix.
        StringValues::Prefix(_) => json!({
            "prefix": {
                field: {
                    "value": value,
                    "case_insensitive": true
                }
            }
        }),
        // Case-sensitive equality.
        StringValues::Exact(_) => json!({
            "match_phrase": {
                field: {
                    "query": value,
                    "analyzer": "keyword"
                }
            }
        }),
        // Case-insensitive substring. The value is escaped so that `*` and
        // `?` typed by the caller match themselves instead of acting as
        // further wildcards.
        StringValues::Contains(_) => json!({
            "wildcard": {
                field: {
                    "value": format!("*{}*", escape_wildcard(value)),
                    "case_insensitive": true
                }
            }
        }),
        // Needs an analysed text field.
        StringValues::Text(_) => {
            return Err(QueryBuildError::unsupported_modifier(values.modifier()));
        }
    })
}

/// Escapes the characters Elasticsearch's `wildcard` query treats specially
/// (`\`, `*`, `?`), so a search value is matched literally.
fn escape_wildcard(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        if matches!(character, '\\' | '*' | '?') {
            escaped.push('\\');
        }
        escaped.push(character);
    }
    escaped
}
