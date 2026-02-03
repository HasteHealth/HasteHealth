use crate::elastic_search::search::QueryBuildError;
use haste_fhir_client::url::Parameter;
use haste_fhir_model::r4::generated::resources::SearchParameter;
use serde_json::json;

pub fn string(
    parsed_parameter: &Parameter,
    search_param: &SearchParameter,
) -> Result<serde_json::Value, QueryBuildError> {
    match parsed_parameter.modifier.as_ref().map(|m| m.as_str()) {
        Some("exact") => {
            let string_params = parsed_parameter
                .value
                .iter()
                .map(|value| {
                    Ok(json!({
                        "match_phrase":{
                            search_param.url.value.as_ref().unwrap(): {
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
        Some(modifier) => Err(QueryBuildError::UnsupportedModifier(modifier.to_string())),
        None => {
            let string_params = parsed_parameter
                .value
                .iter()
                .map(|value| {
                    Ok(json!({
                        "prefix":{
                            search_param.url.value.as_ref().unwrap(): {
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
