use std::sync::Arc;

use crate::{
    ParameterLevel, ResolvedParameter, SearchEntry, SearchOptions, SearchParameterResolve,
    SearchReturn,
    elastic_search::{
        DYNAMIC_PARAMETER_INDEX_FIELD, ElasticSearchResponse, SearchError, get_index_name,
    },
};
use elasticsearch::{Elasticsearch, SearchParts};
use haste_fhir_client::{
    request::SearchRequest,
    url::{Parameter, ParsedParameter, ParsedParameters},
};
use haste_fhir_model::r4::generated::{
    resources::{ResourceType, SearchParameter},
    terminology::SearchParamType,
};
use haste_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use haste_jwt::{ProjectId, TenantId};
use haste_repository::types::SupportedFHIRVersions;
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
        diagnostic = "Unsupported modifier parameter: '{arg0}'"
    )]
    UnsupportedModifier(String),
    #[error(
        code = "not-supported",
        diagnostic = "Prefix '{arg0}' is not supported for this search type."
    )]
    UnsupportedPrefix(String),

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

    match search_param.type_.as_ref() {
        SearchParamType::Date(_) => match direction {
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
        SearchParamType::String(_) => match direction {
            SortDirection::Asc => Ok(json!({
                url: {
                    "order": "asc"
                }
            })),
            SortDirection::Desc => Ok(json!({
                url: {
                    "order": "desc"
                }
            })),
        },
        SearchParamType::Token(_) => match direction {
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

// Handles :missing modifier for string,number,uri which have no nesting. For other modifiers, they are handled in their respective clause functions.
fn simple_missing_modifier(
    search_param: &SearchParameter,
    parsed_parameter: &Parameter,
) -> Result<serde_json::Value, QueryBuildError> {
    if matches!(
        search_param.type_.as_ref(),
        SearchParamType::Composite(None)
    ) {
        return Err(QueryBuildError::UnsupportedModifier("missing".to_string()));
    }

    let url = search_param
        .url
        .value
        .as_ref()
        .map(|v| v.as_str())
        .unwrap_or_default();

    let field_name = match search_param.type_.as_ref() {
        SearchParamType::Uri(_) | SearchParamType::String(_) | SearchParamType::Number(_) => url,
        _ => {
            return Err(QueryBuildError::UnsupportedModifier("missing".to_string()));
        }
    };

    match parsed_parameter.value.as_slice() {
        [v] => match v.as_str() {
            "false" => Ok(json!({
                "exists": {
                    "field": field_name
                }
            })),
            "true" => Ok(json!({
                "bool": {
                    "must_not": {
                        "exists": {
                            "field": field_name
                        }
                    }
                }
            })),
            _ => Err(QueryBuildError::InvalidParameterValue(
                parsed_parameter.name.clone(),
            )),
        },
        _ => {
            return Err(QueryBuildError::InvalidParameterValue(
                parsed_parameter.name.clone(),
            ));
        }
    }
}

fn parameter_to_elasticsearch_clauses(
    parameter: &ResolvedParameter,
    parsed_parameter: &Parameter,
) -> Result<serde_json::Value, QueryBuildError> {
    let namespace = match parameter.level {
        ParameterLevel::System => None,
        ParameterLevel::Project => Some(DYNAMIC_PARAMETER_INDEX_FIELD),
    };
    let search_param = parameter.search_parameter.as_ref();
    let elastic_clause = match search_param.type_.as_ref() {
        SearchParamType::Uri(_) => clauses::uri(namespace, parsed_parameter, search_param),
        SearchParamType::Quantity(_) => {
            clauses::quantity(namespace, parsed_parameter, search_param)
        }
        SearchParamType::Reference(_) => {
            clauses::reference(namespace, parsed_parameter, search_param)
        }
        SearchParamType::Date(_) => clauses::date(namespace, parsed_parameter, search_param),
        SearchParamType::Token(_) => clauses::token(namespace, parsed_parameter, search_param),
        SearchParamType::Number(_) => clauses::number(namespace, parsed_parameter, search_param),
        SearchParamType::String(_) => clauses::string(namespace, parsed_parameter, search_param),
        _ => Err(QueryBuildError::UnsupportedParameter(
            search_param.name.value.clone().unwrap_or_default(),
        )),
    }?;

    match parameter.level {
        ParameterLevel::System => Ok(elastic_clause),
        ParameterLevel::Project => Ok(json!({
            "nested": {
                "path": DYNAMIC_PARAMETER_INDEX_FIELD,
                "query": elastic_clause
            }
        })),
    }
}

// Default value for Elasticsearch is 10k
// see index.max_result_window
static ABSOLUTE_MAX: usize = 10_000;
static DEFAULT_MAX_COUNT: usize = 50;

fn get_resource_type<'a>(request: &'a SearchRequest) -> Option<&'a ResourceType> {
    match request {
        SearchRequest::Type(type_search_request) => Some(&type_search_request.resource_type),
        _ => None,
    }
}

fn get_parameters<'a>(request: &'a SearchRequest) -> &'a ParsedParameters {
    match request {
        SearchRequest::Type(type_search_request) => &type_search_request.parameters,
        SearchRequest::System(system_search_request) => &system_search_request.parameters,
    }
}

async fn build_elastic_search_query<ParameterResolver: SearchParameterResolve>(
    parameter_resolver: Arc<ParameterResolver>,
    tenant: &TenantId,
    project: &ProjectId,
    request: &SearchRequest,
    options: &Option<SearchOptions>,
) -> Result<serde_json::Value, OperationOutcomeError> {
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

    for parameter in parameters.parameters().iter() {
        match parameter {
            ParsedParameter::Resource(resource_param) => {
                let parameter = parameter_resolver
                    .by_name(tenant, project, resource_type, &resource_param.name)
                    .await?
                    .ok_or_else(|| {
                        QueryBuildError::MissingParameter(resource_param.name.to_string())
                    })?;
                let clause = parameter_to_elasticsearch_clauses(&parameter, &resource_param)?;
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
                            )
                            .into());
                        }
                    }
                }
                "_sort" => {
                    for sort_param in result_param.value.iter() {
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

                        let parameter = parameter_resolver
                            .by_name(tenant, project, resource_type, parameter_name)
                            .await?
                            .ok_or_else(|| {
                                QueryBuildError::MissingParameter(parameter_name.to_string())
                            })?;

                        sort.push(sort_build(
                            parameter.search_parameter.as_ref(),
                            &sort_direction,
                        )?);
                    }
                }
                _ => {
                    return Err(QueryBuildError::UnsupportedParameter(
                        result_param.name.to_string(),
                    )
                    .into());
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

    // Allow Span of multiple projects for search.
    clauses.push(json!({
        "match": {
            "project": project.as_ref()
        }
    }));

    let query = json!({
        "fields": ["version_id", "id", "resource_type", "project"],
        "size": size,
        "track_total_hits": show_total,
        "_source": false,
        "from": offset,
        "query": {
            "bool": {
                "filter": clauses
            }
        },
        "sort": sort,
    });

    // println!("{}", serde_json::to_string_pretty(&query).unwrap());

    Ok(query)
}

pub async fn execute_search<ParameterResolver: SearchParameterResolve>(
    es: Arc<Elasticsearch>,
    parameter_resolver: Arc<ParameterResolver>,
    fhir_version: &SupportedFHIRVersions,
    tenant: &TenantId,
    project: &ProjectId,
    search_request: &SearchRequest,
    options: &Option<SearchOptions>,
) -> Result<SearchReturn, haste_fhir_operation_error::OperationOutcomeError> {
    let query = build_elastic_search_query(
        parameter_resolver.clone(),
        tenant,
        project,
        &search_request,
        options,
    )
    .await?;

    let search_response = es
        .search(SearchParts::Index(&[get_index_name(&fhir_version)?]))
        .body(query)
        .send()
        .await
        .map_err(SearchError::from)?;

    if !search_response.status_code().is_success() {
        return Err(SearchError::ElasticSearchResponseError(
            search_response.status_code().as_u16(),
        )
        .into());
    }

    let search_results = search_response
        .json::<ElasticSearchResponse>()
        .await
        .map_err(SearchError::from)?;

    Ok(SearchReturn {
        total: search_results.hits.total.as_ref().map(|t| t.value),
        entries: search_results
            .hits
            .hits
            .into_iter()
            .map(|mut hit| SearchEntry {
                id: hit.fields.id.pop().unwrap(),
                resource_type: hit.fields.resource_type.pop().unwrap(),
                version_id: hit.fields.version_id.pop().unwrap(),
                project: hit.fields.project.pop().unwrap(),
            })
            .collect(),
    })
}
