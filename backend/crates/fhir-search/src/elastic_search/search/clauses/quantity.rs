use crate::elastic_search::search::QueryBuildError;
use oxidized_fhir_client::url::{Parameter };
use oxidized_fhir_model::r4::types::SearchParameter;
use serde_json::json;

pub fn quantity( parsed_parameter: &Parameter, search_param: &SearchParameter) -> Result<serde_json::Value, QueryBuildError> {
    let params = parsed_parameter
        .value
        .iter()
        .map(|value| {
            let pieces = value.split('|').collect::<Vec<&str>>();
            match pieces.len() {
                3 => {
                    let parameter_url = search_param.url.value.as_ref().unwrap().to_string();
                    let mut clauses = vec![];
                    
                    let value = pieces.get(0).unwrap_or(&"");
                    let system = pieces.get(1).unwrap_or(&"");
                    let code = pieces.get(2).unwrap_or(&"");

                    if !value.is_empty() {
                        let value = value
                            .parse::<f64>()
                            .map_err(|_e| QueryBuildError::InvalidParameterValue(value.to_string()))?;

                        clauses.push(json!({
                            "range": {
                                search_param.url.value.as_ref().unwrap().to_string() + ".start_value": {
                                    "lte": value
                                },
            
                            }
                        }));

                        clauses.push(json!({
                            "range": {
                                search_param.url.value.as_ref().unwrap().to_string() + ".end_value": {
                                    "gte": value
                                }
                            }
                        }));
                    }

                    // Not sure if should instead just have an or statement for this but than value would not make sense.
                    if !system.is_empty() {
                        clauses.push(json!({
                            "match": {
                                search_param.url.value.as_ref().unwrap().to_string() + ".start_system": {
                                    "query": system
                                }
                            }
                        }));
                        clauses.push(json!({
                            "match": {
                                search_param.url.value.as_ref().unwrap().to_string() + ".end_system": {
                                    "query": system
                                }
                            }
                        }));
                    }

                    if !code.is_empty() {
                        clauses.push(json!({
                            "match": {
                                search_param.url.value.as_ref().unwrap().to_string() + ".start_code": {
                                    "query": code
                                }
                            }
                        }));
                        clauses.push(json!({
                            "match": {
                                search_param.url.value.as_ref().unwrap().to_string() + ".end_code": {
                                    "query": code
                                }
                            }
                        }));
                    }

                    Ok(json!({
                        "nested": {
                            "path": parameter_url,
                            "query": {
                                "bool": {
                                    "must": clauses
                                }
                            }
                        }
                    }))
                }
                4 => {
                    Err(QueryBuildError::UnsupportedParameterValue(value.to_string()))
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