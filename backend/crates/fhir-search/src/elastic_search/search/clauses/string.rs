use crate::{elastic_search::search::clauses::namespace_parameter, query::Modifier};
use haste_fhir_client::url::Parameter;
use haste_fhir_model::r4::generated::resources::SearchParameter;
use serde_json::json;

/// `modifier` is `Exact`, `Contains` or `None` (already validated).
pub fn string(
    namespace: Option<&str>,
    parsed_parameter: &Parameter,
    search_param: &SearchParameter,
    modifier: Modifier,
) -> serde_json::Value {
    let column_name = namespace_parameter(namespace, search_param);

    let string_params = parsed_parameter
        .value
        .iter()
        .map(|value| match modifier {
            // Case-sensitive equality.
            Modifier::Exact => json!({
                "match_phrase": {
                    &column_name: {
                        "query": value,
                        "analyzer": "keyword"
                    }
                }
            }),
            // Case-insensitive substring.
            Modifier::Contains => json!({
                "wildcard": {
                    &column_name: {
                        "value": format!("*{value}*"),
                        "case_insensitive": true
                    }
                }
            }),
            // Default: case-insensitive prefix.
            _ => json!({
                "prefix": {
                    &column_name: {
                        "value": value,
                        "case_insensitive": true
                    }
                }
            }),
        })
        .collect::<Vec<serde_json::Value>>();

    json!({
        "bool": {
            "should": string_params
        }
    })
}
