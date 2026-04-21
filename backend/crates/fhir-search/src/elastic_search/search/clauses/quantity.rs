use crate::elastic_search::search::{QueryBuildError, clauses::namespace_parameter};
use haste_fhir_client::url::Parameter;
use haste_fhir_model::r4::generated::resources::SearchParameter;
use serde_json::json;

pub fn quantity(
    namespace: Option<&str>,
    parsed_parameter: &Parameter,
    search_param: &SearchParameter,
) -> Result<serde_json::Value, QueryBuildError> {
    let column_name = namespace_parameter(namespace, search_param);
    let params = parsed_parameter
        .value
        .iter()
        .map(|value| {
            let pieces = value.split('|').collect::<Vec<&str>>();
            match pieces.len() {
                3 => {
                    let mut clauses = vec![];

                    let value = pieces.get(0).unwrap_or(&"");
                    let system = pieces.get(1).unwrap_or(&"");
                    let code = pieces.get(2).unwrap_or(&"");

                    if !value.is_empty() {
                        let value = value.parse::<f64>().map_err(|_e| {
                            QueryBuildError::InvalidParameterValue(value.to_string())
                        })?;

                        clauses.push(json!({
                            "range": {
                                format!("{}.start_value", column_name): {
                                    "lte": value
                                },

                            }
                        }));

                        clauses.push(json!({
                            "range": {
                                format!("{}.end_value", column_name): {
                                    "gte": value
                                }
                            }
                        }));
                    }

                    // Not sure if should instead just have an or statement for this but than value would not make sense.
                    if !system.is_empty() {
                        clauses.push(json!({
                            "match": {
                                format!("{}.start_system", column_name): {
                                    "query": system
                                }
                            }
                        }));
                        clauses.push(json!({
                            "match": {
                                format!("{}.end_system", column_name): {
                                    "query": system
                                }
                            }
                        }));
                    }

                    if !code.is_empty() {
                        clauses.push(json!({
                            "match": {
                                format!("{}.start_code", column_name): {
                                    "query": code
                                }
                            }
                        }));
                        clauses.push(json!({
                            "match": {
                                format!("{}.end_code", column_name): {
                                    "query": code
                                }
                            }
                        }));
                    }

                    Ok(json!({
                        "nested": {
                            "path": column_name,
                            "query": {
                                "bool": {
                                    "must": clauses
                                }
                            }
                        }
                    }))
                }
                4 => Err(QueryBuildError::UnsupportedParameterValue(
                    value.to_string(),
                )),
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
