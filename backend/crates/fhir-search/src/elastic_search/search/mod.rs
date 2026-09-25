use std::sync::Arc;

use crate::{
    ParameterLevel, ResolvedParameter, SearchEntry, SearchOptions, SearchParameterResolve,
    SearchReturn,
    elastic_search::{
        DYNAMIC_PARAMETER_INDEX_FIELD, ElasticSearchResponse, SearchError,
        flatten_parameter_field_name, get_index_name,
    },
    query::{Modifier, QueryBuildError, parse_modifier},
};
use elasticsearch::{Elasticsearch, SearchParts};
use haste_fhir_client::{
    request::SearchRequest,
    url::{Parameter, ParsedParameter, ParsedParameters},
};
use haste_fhir_model::r4::generated::{
    resources::{ResourceType, SearchParameter},
    terminology::{IssueType, SearchParamType},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId};
use serde::{Deserialize, Serialize};
use serde_json::json;

mod clauses;

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
    let url = flatten_parameter_field_name(&url);

    match &search_param.type_ {
        param_type if param_type == &SearchParamType::date() => match direction {
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
        param_type if param_type == &SearchParamType::string() => match direction {
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
        param_type if param_type == &SearchParamType::token() => match direction {
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
        _ => Err(QueryBuildError::UnsupportedSortParameter(
            search_param.name.value.clone().unwrap_or_default(),
        )),
    }
}

/// Matches documents where the parameter has at least one value. Token, date,
/// quantity and reference are nested objects; the rest are plain fields.
fn has_value(
    namespace: Option<&str>,
    search_param: &SearchParameter,
) -> Result<serde_json::Value, QueryBuildError> {
    let field = clauses::namespace_parameter(namespace, search_param);
    let param_type = &search_param.type_;

    if [
        SearchParamType::token(),
        SearchParamType::date(),
        SearchParamType::quantity(),
        SearchParamType::reference(),
    ]
    .contains(param_type)
    {
        Ok(json!({ "nested": { "path": field, "query": { "match_all": {} } } }))
    } else if [
        SearchParamType::string(),
        SearchParamType::uri(),
        SearchParamType::number(),
    ]
    .contains(param_type)
    {
        Ok(json!({ "exists": { "field": field } }))
    } else {
        Err(QueryBuildError::UnsupportedModifier("missing".to_string()))
    }
}

/// The positive query for a parameter's values. `modifier` only matters to
/// string (`:exact`, `:contains`).
fn type_clause(
    namespace: Option<&str>,
    parsed_parameter: &Parameter,
    search_param: &SearchParameter,
    modifier: Modifier,
) -> Result<serde_json::Value, QueryBuildError> {
    match &search_param.type_ {
        param_type if param_type == &SearchParamType::uri() => {
            Ok(clauses::uri(namespace, parsed_parameter, search_param))
        }
        param_type if param_type == &SearchParamType::quantity() => {
            clauses::quantity(namespace, parsed_parameter, search_param)
        }
        param_type if param_type == &SearchParamType::reference() => {
            clauses::reference(namespace, parsed_parameter, search_param)
        }
        param_type if param_type == &SearchParamType::date() => {
            clauses::date(namespace, parsed_parameter, search_param)
        }
        param_type if param_type == &SearchParamType::token() => {
            clauses::token(namespace, parsed_parameter, search_param)
        }
        param_type if param_type == &SearchParamType::number() => {
            clauses::number(namespace, parsed_parameter, search_param)
        }
        param_type if param_type == &SearchParamType::string() => Ok(clauses::string(
            namespace,
            parsed_parameter,
            search_param,
            modifier,
        )),
        _ => Err(QueryBuildError::UnsupportedParameter(
            search_param.name.value.clone().unwrap_or_default(),
        )),
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

    // `:not` and `:missing=true` are applied as a `must_not` around the whole
    // clause, outside any `nested` query. Negating inside `nested` would match
    // a resource if *any* of its values failed to match, and never match one
    // with no values at all.
    let modifier = parse_modifier(parsed_parameter, &search_param.type_)?;
    let (clause, negate) = match modifier {
        Modifier::Missing(missing) => (has_value(namespace, search_param)?, missing),
        _ => (
            type_clause(namespace, parsed_parameter, search_param, modifier)?,
            modifier == Modifier::Not,
        ),
    };

    let clause = match parameter.level {
        ParameterLevel::System => clause,
        // The `url` match and the type-specific value match must both land
        // inside this one `nested` query's `bool.must`, so Elasticsearch
        // requires them to be satisfied by the *same* `dynamic_parameters`
        // entry rather than by any two entries independently.
        ParameterLevel::Project => {
            let url = search_param.url.value.as_deref().unwrap_or("");
            let url_field = format!("{DYNAMIC_PARAMETER_INDEX_FIELD}.url");
            json!({
                "nested": {
                    "path": DYNAMIC_PARAMETER_INDEX_FIELD,
                    "query": {
                        "bool": {
                            "must": [
                                { "term": { url_field: url } },
                                clause
                            ]
                        }
                    }
                }
            })
        }
    };

    Ok(if negate {
        json!({ "bool": { "must_not": [clause] } })
    } else {
        clause
    })
}

// Default value for Elasticsearch is 10k
// see index.max_result_window
static ABSOLUTE_MAX: u64 = 10_000;
static DEFAULT_MAX_COUNT: u64 = 50;

fn get_resource_type(request: &SearchRequest) -> Option<&ResourceType> {
    match request {
        SearchRequest::Type(type_search_request) => Some(&type_search_request.resource_type),
        SearchRequest::System(_) => None,
    }
}

fn get_parameters(request: &SearchRequest) -> &ParsedParameters {
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
    options: Option<&SearchOptions>,
) -> Result<serde_json::Value, OperationOutcomeError> {
    let resource_type = get_resource_type(request);
    let parameters = get_parameters(request);

    let mut clauses = Vec::new();
    let mut state = ElasticSearchQueryState {
        max_count: get_max_count(options)?,
        offset: 0,
        show_total: false,
        sort: Vec::new(),
    };

    for parameter in parameters.parameters() {
        match parameter {
            ParsedParameter::Resource(resource_param) => {
                let clause = build_resource_clause(
                    &parameter_resolver,
                    tenant,
                    project,
                    resource_type,
                    resource_param,
                )
                .await?;

                clauses.push(clause);
            }
            ParsedParameter::Result(result_param) => {
                handle_result_parameter(
                    &parameter_resolver,
                    tenant,
                    project,
                    resource_type,
                    result_param,
                    &mut state,
                )
                .await?;
            }
        }
    }

    add_context_clauses(&mut clauses, resource_type, tenant, project);

    Ok(build_elastic_query(
        &clauses,
        state.max_count,
        state.show_total,
        state.offset,
        &state.sort,
    ))
}

fn get_max_count(options: Option<&SearchOptions>) -> Result<u64, OperationOutcomeError> {
    if let Some(count_limit) = options.as_ref().and_then(|o| o.count_limit) {
        if count_limit > ABSOLUTE_MAX {
            return Err(OperationOutcomeError::fatal(
                IssueType::too_costly(),
                "Count limit passed exceeds maxiumum allowed by ES".to_string(),
            ));
        }

        Ok(count_limit)
    } else {
        Ok(DEFAULT_MAX_COUNT)
    }
}

async fn build_resource_clause<ParameterResolver: SearchParameterResolve>(
    parameter_resolver: &Arc<ParameterResolver>,
    tenant: &TenantId,
    project: &ProjectId,
    resource_type: Option<&ResourceType>,
    resource_param: &Parameter,
) -> Result<serde_json::Value, OperationOutcomeError> {
    let parameter = parameter_resolver
        .by_name(tenant, project, resource_type, &resource_param.name)
        .await?
        .ok_or_else(|| QueryBuildError::MissingParameter(resource_param.name.clone()))?;

    Ok(parameter_to_elasticsearch_clauses(
        &parameter,
        resource_param,
    )?)
}

struct ElasticSearchQueryState {
    max_count: u64,
    offset: u64,
    show_total: bool,
    sort: Vec<serde_json::Value>,
}

async fn handle_result_parameter<ParameterResolver: SearchParameterResolve>(
    parameter_resolver: &Arc<ParameterResolver>,
    tenant: &TenantId,
    project: &ProjectId,
    resource_type: Option<&ResourceType>,
    result_param: &Parameter,
    state: &mut ElasticSearchQueryState,
) -> Result<(), OperationOutcomeError> {
    match result_param.name.as_str() {
        "_count" => {
            state.max_count = parse_count_parameter(result_param)?;
        }
        "_offset" => {
            state.offset = parse_offset_parameter(result_param)?;
        }
        "_total" => {
            state.show_total = parse_total_parameter(result_param)?;
        }
        "_sort" => {
            build_sort_parameters(
                parameter_resolver,
                tenant,
                project,
                resource_type,
                result_param,
                &mut state.sort,
            )
            .await?;
        }
        "_summary" | "_elements" => {
            // _elements and _summary are handled in middleware, not in the ES query.
        }
        _ => {
            return Err(QueryBuildError::UnsupportedParameter(result_param.name.clone()).into());
        }
    }

    Ok(())
}

fn parse_count_parameter(result_param: &Parameter) -> Result<u64, OperationOutcomeError> {
    let count_parameter_string = result_param.value.first().ok_or_else(|| {
        OperationOutcomeError::error(
            IssueType::required(),
            format!("Missing parameter value: {}", result_param.name),
        )
    })?;

    count_parameter_string.parse::<u64>().map_err(|_| {
        OperationOutcomeError::fatal(
            IssueType::invalid(),
            format!("Invalid _count value: '{count_parameter_string}'. Make sure it's a positive number."),
        )
    })
}

fn parse_offset_parameter(result_param: &Parameter) -> Result<u64, OperationOutcomeError> {
    let offset_param_string = result_param.value.first().ok_or_else(|| {
        OperationOutcomeError::error(
            IssueType::required(),
            format!("Missing parameter value: {}", result_param.name),
        )
    })?;

    offset_param_string.parse::<u64>().map_err(|_| {
        OperationOutcomeError::fatal(
            IssueType::invalid(),
            format!(
                "Invalid _offset value: '{offset_param_string}'. Make sure it's a positive number."
            ),
        )
    })
}

fn parse_total_parameter(result_param: &Parameter) -> Result<bool, OperationOutcomeError> {
    match result_param
        .value
        .iter()
        .map(std::string::String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["none"] => Ok(false),
        ["accurate" | "estimate"] => Ok(true),
        _ => Err(QueryBuildError::InvalidParameterValue(result_param.name.clone()).into()),
    }
}

async fn build_sort_parameters<ParameterResolver: SearchParameterResolve>(
    parameter_resolver: &Arc<ParameterResolver>,
    tenant: &TenantId,
    project: &ProjectId,
    resource_type: Option<&ResourceType>,
    result_param: &Parameter,
    sort: &mut Vec<serde_json::Value>,
) -> Result<(), OperationOutcomeError> {
    for sort_param in &result_param.value {
        let parameter_name = sort_param.strip_prefix('-').unwrap_or(sort_param);

        let sort_direction = if sort_param.starts_with('-') {
            SortDirection::Desc
        } else {
            SortDirection::Asc
        };

        let parameter = parameter_resolver
            .by_name(tenant, project, resource_type, parameter_name)
            .await?
            .ok_or_else(|| QueryBuildError::MissingParameter(parameter_name.to_string()))?;

        sort.push(sort_build(
            parameter.search_parameter.as_ref(),
            &sort_direction,
        )?);
    }

    Ok(())
}

fn add_context_clauses(
    clauses: &mut Vec<serde_json::Value>,
    resource_type: Option<&ResourceType>,
    tenant: &TenantId,
    project: &ProjectId,
) {
    if let Some(resource_type) = resource_type {
        clauses.push(json!({
            "match": {
                "resource_type": resource_type.as_ref()
            }
        }));
    }

    clauses.push(json!({
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
}

fn build_elastic_query(
    clauses: &[serde_json::Value],
    max_count: u64,
    show_total: bool,
    offset: u64,
    sort: &[serde_json::Value],
) -> serde_json::Value {
    json!({
        "fields": ["version_id", "id", "resource_type", "project"],
        "size": max_count,
        "track_total_hits": show_total,
        "_source": false,
        "from": offset,
        "query": {
            "bool": {
                "filter": clauses
            }
        },
        "sort": sort,
    })
}

pub async fn execute_search<ParameterResolver: SearchParameterResolve>(
    es: Arc<Elasticsearch>,
    parameter_resolver: Arc<ParameterResolver>,
    tenant: &TenantId,
    project: &ProjectId,
    search_request: &SearchRequest,
    options: Option<&SearchOptions>,
) -> Result<SearchReturn, haste_fhir_operation_error::OperationOutcomeError> {
    let query = build_elastic_search_query(
        parameter_resolver.clone(),
        tenant,
        project,
        search_request,
        options,
    )
    .await?;

    let search_response = es
        .search(SearchParts::Index(&[get_index_name()]))
        .body(query)
        .send()
        .await
        .map_err(SearchError::from)?;

    if !search_response.status_code().is_success() {
        let response = Err(SearchError::ElasticSearchResponseError(
            search_response.status_code().as_u16(),
        )
        .into());

        return response;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn parameter(level: ParameterLevel, type_: &str) -> ResolvedParameter {
        let search_parameter: SearchParameter = serde_json::from_value(json!({
            "resourceType": "SearchParameter",
            "url": "http://example.org/SearchParameter/p",
            "name": "p",
            "code": "p",
            "status": "active",
            "description": "p",
            "base": ["Observation"],
            "type": type_,
            "expression": "Observation.code"
        }))
        .unwrap();
        ResolvedParameter::new(level, Arc::new(search_parameter))
    }

    fn parsed(modifier: Option<&str>, values: &[&str]) -> Parameter {
        Parameter {
            name: "p".to_string(),
            modifier: modifier.map(str::to_string),
            value: values.iter().map(|v| (*v).to_string()).collect(),
            chains: None,
        }
    }

    fn clause(
        level: ParameterLevel,
        type_: &str,
        modifier: Option<&str>,
        values: &[&str],
    ) -> serde_json::Value {
        parameter_to_elasticsearch_clauses(&parameter(level, type_), &parsed(modifier, values))
            .expect("clause builds")
    }

    /// Negating inside `nested` matches a resource with any non-matching
    /// value; the negation must wrap the whole nested query.
    #[test]
    fn not_is_applied_outside_the_nested_query() {
        let query = clause(ParameterLevel::System, "token", Some("not"), &["s|A"]);
        let negated = &query["bool"]["must_not"][0];
        assert!(
            negated["bool"]["should"][0]["nested"].is_object(),
            "{query}"
        );
    }

    #[test]
    fn missing_is_applied_outside_the_dynamic_parameters_wrapper() {
        let query = clause(
            ParameterLevel::Project,
            "string",
            Some("missing"),
            &["true"],
        );
        let nested = &query["bool"]["must_not"][0]["nested"];
        assert_eq!(nested["path"], DYNAMIC_PARAMETER_INDEX_FIELD, "{query}");
        assert_eq!(
            nested["query"]["bool"]["must"][1]["exists"]["field"],
            "dynamic_parameters.value.string",
            "{query}"
        );
    }

    #[test]
    fn missing_false_on_a_nested_type_requires_a_nested_value() {
        let query = clause(
            ParameterLevel::System,
            "reference",
            Some("missing"),
            &["false"],
        );
        assert!(query["nested"]["query"]["match_all"].is_object(), "{query}");
    }

    /// `system|` once also required an empty code, so it matched nothing.
    #[test]
    fn a_trailing_pipe_matches_on_system_alone() {
        let query = clause(ParameterLevel::System, "token", None, &["http://sys|"]);
        let inner = &query["bool"]["should"][0]["nested"]["query"];
        assert!(
            inner["match"]
                .as_object()
                .unwrap()
                .keys()
                .all(|k| k.ends_with(".system")),
            "{query}"
        );
    }

    #[test]
    fn a_leading_pipe_requires_no_system() {
        let query = clause(ParameterLevel::System, "token", None, &["|code"]);
        let inner = &query["bool"]["should"][0]["nested"]["query"]["bool"];
        assert!(inner["must_not"][0]["exists"].is_object(), "{query}");
    }

    #[test]
    fn a_bare_quantity_value_matches_any_unit() {
        let query = clause(ParameterLevel::System, "quantity", None, &["70.2"]);
        let must = query["bool"]["should"][0]["nested"]["query"]["bool"]["must"]
            .as_array()
            .unwrap()
            .clone();
        assert_eq!(must.len(), 2, "only the value range: {query}");
    }

    #[test]
    fn unsupported_modifiers_are_rejected() {
        for (type_, modifier) in [("reference", "foo"), ("date", "exact"), ("number", "not")] {
            assert!(
                parameter_to_elasticsearch_clauses(
                    &parameter(ParameterLevel::System, type_),
                    &parsed(Some(modifier), &["1"]),
                )
                .is_err(),
                "{type_}:{modifier}"
            );
        }
    }
}
