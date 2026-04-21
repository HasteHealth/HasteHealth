use crate::elastic_search::search::{
    QueryBuildError, clauses::namespace_parameter, simple_missing_modifier,
};
use haste_fhir_client::url::Parameter;
use haste_fhir_model::r4::generated::resources::SearchParameter;
use serde_json::json;

pub fn string(
    namespace: Option<&str>,
    parsed_parameter: &Parameter,
    search_param: &SearchParameter,
) -> Result<serde_json::Value, QueryBuildError> {
    let column_name = namespace_parameter(namespace, search_param);

    match parsed_parameter.modifier.as_ref().map(|m| m.as_str()) {
        Some("missing") => simple_missing_modifier(search_param, parsed_parameter),
        Some("exact") => {
            let string_params = parsed_parameter
                .value
                .iter()
                .map(|value| {
                    Ok(json!({
                        "match_phrase":{
                            &column_name: {
                                "query": value,
                                "analyzer": "keyword"
                            }
                        }
                    }))
                })
                .collect::<Result<Vec<serde_json::Value>, QueryBuildError>>()?;

            Ok(json!({
                "bool": {
                    "should": string_params
                }
            }))
        }
        Some("contains") => {
            let string_params = parsed_parameter
                .value
                .iter()
                .map(|value| {
                    Ok(json!({
                        "wildcard":{
                            &column_name: {
                                "value": format!("*{}*", value),
                                "case_insensitive": true
                            }
                        }
                    }))
                })
                .collect::<Result<Vec<serde_json::Value>, QueryBuildError>>()?;

            Ok(json!({
                "bool": {
                    "should": string_params
                }
            }))
        }
        Some(modifier) => Err(QueryBuildError::UnsupportedModifier(modifier.to_string())),
        None => {
            let string_params = parsed_parameter
                .value
                .iter()
                .map(|value| {
                    Ok(json!({
                        "prefix":{
                            &column_name: {
                                "value": value,
                                "case_insensitive": true
                            }
                        }
                    }))
                })
                .collect::<Result<Vec<serde_json::Value>, QueryBuildError>>()?;

            Ok(json!({
                "bool": {
                    "should": string_params
                }
            }))
        }
    }
}
