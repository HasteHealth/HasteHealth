use oxidized_fhir_client::url::{Parameter, ParsedParameter};
use oxidized_fhir_model::r4::types::{ResourceType, SearchParameter};
use oxidized_fhir_operation_error::derive::OperationOutcomeError;
use oxidized_fhir_repository::VersionIdRef;
use serde_json::json;

use crate::SearchRequest;

#[derive(OperationOutcomeError, Debug)]
pub enum QueryBuildError {
    #[error(
        code = "not-found",
        diagnostic = "Search parameter with name '{arg1}' not found for resource type '{arg0:?}'"
    )]
    MissingParameter(ResourceType, String),
    #[error(
        code = "invalid",
        diagnostic = "Unsupported search request type or parameter: {arg0}"
    )]
    UnsupportedParameter(String),
    #[error(code = "invalid", diagnostic = "Invalid parameter value: '{arg0}'")]
    InvalidParameterValue(String),
}

fn parameter_to_elasticsearch_clauses(
    search_param: &SearchParameter,
    parsed_parameter: &Parameter,
) -> Result<serde_json::Value, QueryBuildError> {
    match search_param.type_.value.as_ref().map(|s| s.as_str()) {
        Some("token") => {
            let params = parsed_parameter
                .value
                .iter()
                .map(|value| {
                    let pieces = value.split('|').collect::<Vec<&str>>();

                    let k = json!({
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
                    });

                    Ok(k)
                })
                .collect::<Result<Vec<serde_json::Value>, QueryBuildError>>()?;

            Ok(json!({
                "bool": {
                    "should": params
                }
            }))
        }
        Some("number") => {
            let params = parsed_parameter
                .value
                .iter()
                .map(|value| {
                    let v = value
                        .parse::<f64>()
                        .map_err(|_e| QueryBuildError::InvalidParameterValue(value.to_string()))?;
                    let k = json!({
                        "match": {
                            "query": {
                                "range": {
                                    search_param.url.value.as_ref().unwrap(): {
                                        "gte": v,
                                        "lte": v
                                    }
                                }
                            }
                        }
                    });

                    Ok(k)
                })
                .collect::<Result<Vec<serde_json::Value>, QueryBuildError>>()?;

            Ok(json!({
                "bool": {
                    "should": params
                }
            }))
        }
        Some("string") => {
            let string_params = parsed_parameter
                .value
                .iter()
                .map(|value| {
                    Ok(json!({
                        "match":{
                            search_param.url.value.as_ref().unwrap(): {
                                "query": value
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
        _ => todo!(),
    }
}

pub fn build_elastic_search_query(
    request: &SearchRequest,
) -> Result<serde_json::Value, QueryBuildError> {
    match request {
        SearchRequest::TypeSearch(type_search_request) => {
            let resource_type = &type_search_request.resource_type;
            let parameters = &type_search_request.parameters;

            let mut clauses: Vec<serde_json::Value> = vec![];

            for parameter in parameters.iter() {
                match parameter {
                    ParsedParameter::Resource(resource_param) => {
                        let search_param =
                            oxidized_artifacts::search_parameters::get_search_parameter_for_name(
                                &resource_type,
                                &resource_param.name,
                            )
                            .ok_or_else(|| {
                                QueryBuildError::MissingParameter(
                                    resource_type.clone(),
                                    resource_param.name.to_string(),
                                )
                            })?;
                        let clause =
                            parameter_to_elasticsearch_clauses(&search_param, &resource_param)?;
                        clauses.push(clause);
                    }
                    ParsedParameter::Result(result_param) => {
                        return Err(QueryBuildError::UnsupportedParameter(
                            result_param.name.to_string(),
                        ));
                    }
                }
            }

            clauses.push(json!({
                "match": {
                    "resource_type": resource_type.as_str()
                }
            }));

            let query = json!({
                "query": {
                    "bool": {
                        "must": clauses
                    }
                },
                "size": 100,
                "fields": ["version_id", "id"],
                "_source": false,
            });

            Ok(query)
        }
        _ => {
            return Err(QueryBuildError::UnsupportedParameter(
                "Unsupported search request type".to_string(),
            ));
        }
    }
}
