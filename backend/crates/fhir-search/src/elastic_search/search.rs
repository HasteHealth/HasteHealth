use oxidized_fhir_client::url::{Parameter, ParsedParameter};
use oxidized_fhir_model::r4::{
    datetime::parse_datetime,
    types::{ResourceType, SearchParameter},
};
use oxidized_fhir_operation_error::derive::OperationOutcomeError;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{indexing_conversion::{date_time_range, get_decimal_range}, SearchRequest};

#[derive(OperationOutcomeError, Debug)]
pub enum QueryBuildError {
    #[error(
        code = "not-found",
        diagnostic = "Search parameter with name '{arg0}' not found.'"
    )]
    MissingParameter(String),
    #[error(
        code = "not-supported",
        diagnostic = "Unsupported parameter: '{arg0}'"
    )]
    UnsupportedParameter(String),
        #[error(
        code = "not-supported",
        diagnostic = "Unsupported sorting parameter: '{arg0}'"
    )]
    UnsupportedSortParameter(String),
        #[error(
        code = "not-supported",
        diagnostic = "Parameter value '{arg0}' is not supported for this search type."
    )]
    UnsupportedParameterValue(String),
    #[error(code = "invalid", diagnostic = "Invalid parameter value: '{arg0}'")]
    InvalidParameterValue(String),
    #[error(code = "invalid", diagnostic = "Invalid date format: '{arg0}'")]
    InvalidDateFormat(String),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum SortDirection {
    Asc,
    Desc,
}

fn sort_build(search_param: &SearchParameter, direction: &SortDirection) -> Result<serde_json::Value, QueryBuildError> {
    let url = search_param.url.value.clone().ok_or_else(|| {
        QueryBuildError::UnsupportedParameter(search_param.name.value.clone().unwrap_or_default())
    })?;

    match search_param.type_.value.as_ref().map(|s| s.as_str()) {
        Some("date") => {
            match direction {
                SortDirection::Asc => {
                    let sort_col = url.clone() + ".start";
                    Ok(json!({
                        sort_col: {
                            "order": "asc",
                            "nested": {
                                "path": url
                            }
                        }
                    }))
                }
                SortDirection::Desc => {
                    let sort_col = url.clone() + ".end";
                    Ok(json!({
                        sort_col: {
                            "order": "desc",
                            "nested": {
                                "path": url
                            }
                        }
                    }))
                }
            }
        }
        Some("string") => {
            match direction {
                SortDirection::Asc => {
                    let sort_col = url.clone();
                    Ok(json!({
                        sort_col: {
                            "order": "asc"
                        }
                    }))
                }
                SortDirection::Desc => {
                    let sort_col = url.clone();
                    Ok(json!({
                        sort_col: {
                            "order": "desc"
                        }
                    }))
                }
            }
        }
        _ => return Err(QueryBuildError::UnsupportedSortParameter(search_param.name.value.clone().unwrap_or_default())),
    }
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

                    let range = get_decimal_range(v);

                    let k = json!({
                        "range": {
                            search_param.url.value.as_ref().unwrap(): {
                                "gte": range.start,
                                "lte": range.end
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
        _ => todo!(),
    }
}

static MAX_COUNT: usize = 50;

fn get_resource_type<'a>(request: &'a SearchRequest) -> Option<&'a ResourceType> {
    match request {
        SearchRequest::TypeSearch(type_search_request) => Some(&type_search_request.resource_type),
        _ => None,
    }
}

fn get_parameters<'a>(request: &'a SearchRequest) -> &'a Vec<ParsedParameter> {
    match request {
        SearchRequest::TypeSearch(type_search_request) => &type_search_request.parameters,
        SearchRequest::SystemSearch(system_search_request) => &system_search_request.parameters,
    }
}

pub fn build_elastic_search_query(
    request: &SearchRequest,
) -> Result<serde_json::Value, QueryBuildError> {
    let resource_type = get_resource_type(request);
    let parameters = get_parameters(request);

    let mut clauses: Vec<serde_json::Value> = vec![];
    let mut size = MAX_COUNT;
    let mut show_total = false;
    let mut sort: Vec<serde_json::Value> = Vec::new();

    for parameter in parameters.iter() {
        match parameter {
            ParsedParameter::Resource(resource_param) => {
                let search_param =
                    oxidized_artifacts::search_parameters::get_search_parameter_for_name(
                        resource_type,
                        &resource_param.name,
                    )
                    .ok_or_else(|| {
                        QueryBuildError::MissingParameter(
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
                    "_sort" => {
                        sort = result_param.value.iter().map(|sort_param| {
                            let parameter_name = if sort_param.starts_with("-") {
                                &sort_param[1..]
                            } else {
                                sort_param
                            };

                            let sort_direction = if sort_param.starts_with("-") {
                                SortDirection::Desc
                            } else {
                                SortDirection::Asc
                            };

                            let search_param =
                                oxidized_artifacts::search_parameters::get_search_parameter_for_name(
                                    resource_type,
                                    parameter_name,
                                )
                                .ok_or_else(|| {
                                    QueryBuildError::MissingParameter(
                                        parameter_name.to_string(),
                                    )
                            })?;

                            sort_build(search_param.as_ref(), &sort_direction)
                        }).collect::<Result<Vec<_>, _>>()?;
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

    if let Some(resource_type) = resource_type {
        clauses.push(json!({
            "match": {
                "resource_type": resource_type.as_str()
            }
        }));
    }

    let query = json!({
        "fields": ["version_id", "id", "resource_type"],
        "size": size,
        "track_total_hits": show_total,
        "_source": false,
        "query": {
            "bool": {
                "must": clauses
            }
        },
        "sort": sort,
    });

    println!("{}", serde_json::to_string_pretty(&query).unwrap());

    Ok(query)
}
