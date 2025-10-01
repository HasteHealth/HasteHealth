use crate::elastic_search::search::QueryBuildError;
use oxidized_fhir_client::url::Parameter;
use oxidized_fhir_model::r4::generated::resources::SearchParameter;
use serde_json::json;

fn matching_modifier(modifier: &Option<String>) -> Result<String, QueryBuildError> {
    match modifier.as_ref().map(|s| s.as_str()) {
        Some("not") => Ok("must_not".to_string()),
        Some(modifier) => Err(QueryBuildError::ModifierNotSupported(modifier.into())),
        None => Ok("must".to_string()),
    }
}

pub fn token(
    parameter: &Parameter,
    search_param: &SearchParameter,
) -> Result<serde_json::Value, QueryBuildError> {
    let matching_type = matching_modifier(&parameter.modifier)?;
    let params = parameter
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
                                "bool": {
                                    matching_type.clone(): [{
                                        "match": {
                                            search_param.url.value.as_ref().unwrap().to_string() + ".code": {
                                            "query": pieces.get(0)
                                        }
                                    }}]
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
                                    matching_type.clone(): [{
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
                                    }]
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
