use oxidized_fhir_client::url::{Parameter, ParsedParameter};
use oxidized_fhir_model::r4::{
    datetime::parse_datetime,
    types::{ResourceType, SearchParameter},
};
use oxidized_fhir_operation_error::derive::OperationOutcomeError;
use serde_json::json;

use crate::{indexing_conversion::date_time_range, SearchRequest};

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
    #[error(code = "invalid", diagnostic = "Invalid date format: '{arg0}'")]
    InvalidDateFormat(String),
}

fn parameter_to_elasticsearch_clauses(
    search_param: &SearchParameter,
    parsed_parameter: &Parameter,
) -> Result<serde_json::Value, QueryBuildError> {
    match search_param.type_.value.as_ref().map(|s| s.as_str()) {
        Some("uri") => {
            let uri_params = parsed_parameter
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
                    "should": uri_params
                }
            }))
        }
        Some("quantity") => {
            Err(QueryBuildError::UnsupportedParameter(
                "Quantity search parameters are not supported yet".to_string(),
            ))
        }
        Some("reference") => {
            let params = parsed_parameter
                .value
                .iter()
                .map(|value| {
                    let pieces = value.split('/').collect::<Vec<&str>>();
                    match pieces.len() {
                        1 => {
                            Ok(json!({
                                "nested": {
                                    "path": search_param.url.value.as_ref().unwrap(),
                                    "query": {
                                        "match": {
                                            search_param.url.value.as_ref().unwrap().to_string() + ".id": {
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
                                                        search_param.url.value.as_ref().unwrap().to_string() + ".resource_type": {
                                                            "query": pieces.get(0)
                                                        }
                                                    }
                                                },
                                                {
                                                    "match": {
                                                        search_param.url.value.as_ref().unwrap().to_string() + ".id": {
                                                            "query": pieces.get(1)
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
        Some("date") => {
            let params = parsed_parameter
                .value
                .iter()
                .map(|value| {
                    let date_time = parse_datetime(value).map_err(|_e| 
                        QueryBuildError::InvalidDateFormat(value.to_string()))?;
                    let date_range = date_time_range(&date_time).map_err(|_e| 
                        QueryBuildError::InvalidDateFormat(value.to_string()))?;

                    Ok(json!({
                        "nested": {
                            "path": search_param.url.value.as_ref().unwrap(),
                            "query": {
                                "bool": {
                                    "must": [
                                        {
                                            "range": {
                                                search_param.url.value.as_ref().unwrap().to_string() + ".start": {
                                                    "gte": date_range.start
                                                }
                                            }
                                        },
                                        {
                                            "range": {
                                                search_param.url.value.as_ref().unwrap().to_string() + ".end": {
                                                    "lte": date_range.end
                                                }
                                            }
                                        }
                                    ]
                                }
                            }
                        }
                    }))
                })
                .collect::<Result<Vec<serde_json::Value>, QueryBuildError>>()?;

            Ok(json!({
                "bool": {
                    "should": params
                }
            }))
        }
        Some("token") => {
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

static MAX_COUNT: usize = 50;

pub fn build_elastic_search_query(
    request: &SearchRequest,
) -> Result<serde_json::Value, QueryBuildError> {
    match request {
        SearchRequest::TypeSearch(type_search_request) => {
            let resource_type = &type_search_request.resource_type;
            let parameters = &type_search_request.parameters;

            let mut clauses: Vec<serde_json::Value> = vec![];
            let mut size = MAX_COUNT;
            let mut show_total = false;

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
                        match result_param.name.as_str() {
                            "_count" => {
                                size = std::cmp::min( result_param
                                    .value
                                    .get(0)
                                    .and_then(|v| v.parse::<usize>().ok())
                                    .unwrap_or(100), MAX_COUNT);
                            }
                            "_total" => {
                                match result_param.value.iter().map(|s| s.as_str()).collect::<Vec<_>>().as_slice() {
                                    ["none"] => {
                                        show_total = false;
                                    }
                                    ["accurate"] => {
                                        show_total = true;
                                    }
                                    ["estimate"] => {
                                        show_total = true;
                                    }
                                    _ => {
                                        return Err(QueryBuildError::InvalidParameterValue(
                                            result_param.name.to_string(),
                                        ));
                                    }
                                }
                            }
                            _ => {
                                return Err(QueryBuildError::UnsupportedParameter(
                                  result_param.name.to_string(),
                                ));
                            }
                        }
                    }
                }
            }

            clauses.push(json!({
                "match": {
                    "resource_type": resource_type.as_str()
                }
            }));

            let query = json!({
                "fields": ["version_id", "id"],
                "size": size,
                "track_total_hits": show_total,
                "_source": false,
                "query": {
                    "bool": {
                        "must": clauses
                    }
                }
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
