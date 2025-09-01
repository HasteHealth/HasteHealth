use crate::{SearchOptions, SearchRequest};
use oxidized_fhir_client::url::{Parameter, ParsedParameter};
use oxidized_fhir_model::r4::types::{ResourceType, SearchParameter};
use oxidized_fhir_operation_error::derive::OperationOutcomeError;
use oxidized_repository::types::{ProjectId, TenantId};
use serde::{Deserialize, Serialize};
use serde_json::json;

mod clauses;

#[derive(OperationOutcomeError, Debug)]
pub enum QueryBuildError {
    #[error(
        code = "not-found",
        diagnostic = "Search parameter with name '{arg0}' not found.'"
    )]
    MissingParameter(String),
    #[error(code = "not-supported", diagnostic = "Unsupported parameter: '{arg0}'")]
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
    #[error(
        code = "not-supported",
        diagnostic = "Modifier '{arg0}' is not supported"
    )]
    ModifierNotSupported(String),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum SortDirection {
    Asc,
    Desc,
}

fn sort_build(
    search_param: &SearchParameter,
    direction: &SortDirection,
) -> Result<serde_json::Value, QueryBuildError> {
    let url = search_param.url.value.clone().ok_or_else(|| {
        QueryBuildError::UnsupportedParameter(search_param.name.value.clone().unwrap_or_default())
    })?;

    match search_param.type_.value.as_ref().map(|s| s.as_str()) {
        Some("date") => match direction {
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
        },
        Some("string") => match direction {
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
        },
        Some("token") => match direction {
            SortDirection::Asc => {
                let sort_col = url.clone() + ".code";
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
                let sort_col = url.clone() + ".code";
                Ok(json!({
                    sort_col: {
                        "order": "desc",
                        "nested": {
                            "path": url
                        }
                    }
                }))
            }
        },
        _ => {
            return Err(QueryBuildError::UnsupportedSortParameter(
                search_param.name.value.clone().unwrap_or_default(),
            ));
        }
    }
}

fn parameter_to_elasticsearch_clauses(
    search_param: &SearchParameter,
    parsed_parameter: &Parameter,
) -> Result<serde_json::Value, QueryBuildError> {
    match search_param.type_.value.as_ref().map(|s| s.as_str()) {
        Some("uri") => clauses::uri(parsed_parameter, search_param),
        Some("quantity") => clauses::quantity(parsed_parameter, search_param),
        Some("reference") => clauses::reference(parsed_parameter, search_param),
        Some("date") => clauses::date(parsed_parameter, search_param),
        Some("token") => clauses::token(parsed_parameter, search_param),
        Some("number") => clauses::number(parsed_parameter, search_param),
        Some("string") => clauses::string(parsed_parameter, search_param),
        _ => todo!(),
    }
}

// Default value for Elasticsearch is 10k
// see index.max_result_window
static ABSOLUTE_MAX: usize = 10_000;
static DEFAULT_MAX_COUNT: usize = 50;

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
    tenant: &TenantId,
    project: &ProjectId,
    request: &SearchRequest,
    options: &Option<SearchOptions>,
) -> Result<serde_json::Value, QueryBuildError> {
    let resource_type = get_resource_type(request);
    let parameters = get_parameters(request);

    let mut clauses: Vec<serde_json::Value> = vec![];
    let mut size = if let Some(options) = options
        && !options.count_limit
    {
        ABSOLUTE_MAX
    } else {
        DEFAULT_MAX_COUNT
    };
    let mut show_total = false;
    let mut sort: Vec<serde_json::Value> = Vec::new();
    let mut offset: usize = 0;

    for parameter in parameters.iter() {
        match parameter {
            ParsedParameter::Resource(resource_param) => {
                let search_param =
                    oxidized_artifacts::search_parameters::get_search_parameter_for_name(
                        resource_type,
                        &resource_param.name,
                    )
                    .ok_or_else(|| {
                        QueryBuildError::MissingParameter(resource_param.name.to_string())
                    })?;
                let clause = parameter_to_elasticsearch_clauses(&search_param, &resource_param)?;
                clauses.push(clause);
            }
            ParsedParameter::Result(result_param) => match result_param.name.as_str() {
                "_count" => {
                    size = std::cmp::min(
                        result_param
                            .value
                            .get(0)
                            .and_then(|v| v.parse::<usize>().ok())
                            .unwrap_or(100),
                        DEFAULT_MAX_COUNT,
                    );
                }
                "_offset" => {
                    offset = std::cmp::max(
                        result_param
                            .value
                            .get(0)
                            .and_then(|v| v.parse::<usize>().ok())
                            .unwrap_or(0),
                        0,
                    );
                }
                "_total" => {
                    match result_param
                        .value
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .as_slice()
                    {
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
            },
        }
    }

    if let Some(resource_type) = resource_type {
        clauses.push(json!({
            "match": {
                "resource_type": resource_type.as_ref()
            }
        }));
    }

    clauses.push(json! ({
        "match": {
            "tenant": tenant.as_ref()
        }
    }));

    clauses.push(json! ({
        "match": {
            "project": project.as_ref()
        }
    }));

    let query = json!({
        "fields": ["version_id", "id", "resource_type"],
        "size": size,
        "track_total_hits": show_total,
        "_source": false,
        "from": offset,
        "query": {
            "bool": {
                "must": clauses
            }
        },
        "sort": sort,
    });

    // println!("{}", serde_json::to_string_pretty(&query).unwrap());

    Ok(query)
}
