use crate::elastic_search::search::QueryBuildError;
use oxidized_fhir_client::url::Parameter;
use oxidized_fhir_model::r4::types::SearchParameter;
use serde_json::json;

pub fn token(
    parsed_parameter: &Parameter,
    search_param: &SearchParameter,
) -> Result<serde_json::Value, QueryBuildError> {
    let params = parsed_parameter
        .value
        .iter()
        .map(|value| {
            let pieces = value.split('|').collect::<Vec<&str>>();
            match pieces.len() {
                1 => {
                    Ok(json!({
                        "nested": {
                            "path": search_param.url.value.as_ref().unwrap(),
                            "query": {
                                "match": {
                                    search_param.url.value.as_ref().unwrap().to_string() + ".code": {
                                    "query": pieces.get(0)
                                    }
                                }
                            }
                        }
                    }))
                }
                2 => {
                    Ok(json!({
                        "nested": {
                            "path": search_param.url.value.as_ref().unwrap(),
                            "query": {
                                "bool": {
                                    "must": [
                                        {
                                            "match": {
                                                search_param.url.value.as_ref().unwrap().to_string() + ".code": {
                                                    "query": pieces.get(1)
                                                }
                                            }
                                        },
                                        {
                                            "match": {
                                                search_param.url.value.as_ref().unwrap().to_string() + ".system": {
                                                    "query": pieces.get(0)
                                                }
                                            }
                                        }
                                    ]
                                }
                            }
                        }
                    }))
                }
                _ => Err(QueryBuildError::InvalidParameterValue(value.to_string())),
            }
        })
        .collect::<Result<Vec<serde_json::Value>, QueryBuildError>>()?;

    Ok(json!({
        "bool": {
            "should": params
        }
    }))
}
