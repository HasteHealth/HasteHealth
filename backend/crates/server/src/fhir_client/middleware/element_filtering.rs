use crate::fhir_client::{
    ServerCTX,
    middleware::{
        ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput,
        ServerMiddlewareState,
    },
};
use haste_fhir_client::{
    FHIRClient,
    middleware::MiddlewareChain,
    request::{FHIRRequest, FHIRResponse, SearchRequest, SearchResponse},
    url::{ParsedParameter, ParsedParameters},
};
use haste_fhir_model::r4::generated::{
    resources::{Bundle, Resource},
    terminology::IssueType,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::Repository;
use std::sync::Arc;

/// Element Filtering (should occur after access control checks).
pub struct Middleware {}
impl Middleware {
    pub fn new() -> Self {
        Middleware {}
    }
}

fn find_element_summary_parameter(parameters: &ParsedParameters) -> Option<Vec<&ParsedParameter>> {
    let matched: Vec<&ParsedParameter> = parameters
        .parameters()
        .iter()
        .filter(|p| match p {
            ParsedParameter::Result(result_param) => {
                matches!(result_param.name.as_str(), "_elements" | "_summary")
            }
            ParsedParameter::Resource(_) => false,
        })
        .collect();

    if matched.is_empty() {
        None
    } else {
        Some(matched)
    }
}

pub fn get_summary_element_parameter(request: &FHIRRequest) -> Option<Vec<&ParsedParameter>> {
    match request {
        FHIRRequest::Search(search_request) => match search_request {
            SearchRequest::System(system_search_request) => {
                find_element_summary_parameter(&system_search_request.parameters)
            }
            SearchRequest::Type(type_search_request) => {
                find_element_summary_parameter(&type_search_request.parameters)
            }
        },
        _ => None,
    }
}

/// `_elements` is a comma-separated list of top-level element names and may be repeated.
/// See the following for more information <https://hl7.org/fhir/R4/search.html#elements>.
fn requested_elements<'a>(parameters: &'a [&ParsedParameter]) -> Vec<&'a str> {
    parameters
        .iter()
        .filter_map(|p| match p {
            ParsedParameter::Result(param) if param.name == "_elements" => Some(param),
            _ => None,
        })
        .flat_map(|param| param.value.iter().map(std::string::String::as_str))
        .collect()
}

/// See <https://hl7.org/fhir/R4/search.html#summary>. `Data` is not yet
/// implemented: unlike `True`/`Text`, "everything except `text`" is a
/// denylist rather than an allowlist. Treated as a no-op for now, same as `False`.
enum SummaryMode {
    True,
    Text,
    Data,
    Count,
    False,
}

impl From<&str> for SummaryMode {
    fn from(value: &str) -> Self {
        match value {
            "true" => SummaryMode::True,
            "text" => SummaryMode::Text,
            "data" => SummaryMode::Data,
            "count" => SummaryMode::Count,
            _ => SummaryMode::False,
        }
    }
}

enum Subsetting<'a> {
    Elements(Vec<&'a str>),
    Summary(SummaryMode),
}

/// `_elements` takes precedence over `_summary` when both are present; the
/// spec doesn't define combined semantics for using both at once.
fn subsetting<'a>(parameters: &'a [&ParsedParameter]) -> Option<Subsetting<'a>> {
    let elements = requested_elements(parameters);

    if !elements.is_empty() {
        return Some(Subsetting::Elements(elements));
    }

    parameters.iter().find_map(|p| match p {
        ParsedParameter::Result(param) if param.name == "_summary" => param
            .value
            .first()
            .map(|value| Subsetting::Summary(SummaryMode::from(value.as_str()))),
        _ => None,
    })
}

fn filter_resource(
    resource: Resource,
    subsetting: &Subsetting,
) -> Result<Resource, OperationOutcomeError> {
    let fields: &[&str] = match subsetting {
        Subsetting::Elements(fields) => fields,
        Subsetting::Summary(SummaryMode::True) => &[],
        Subsetting::Summary(SummaryMode::Text) => &["text"],
        // Count is handled at the bundle level (no entries at all); Data
        // isn't implemented yet. Both pass the resource through unchanged.
        Subsetting::Summary(SummaryMode::Data | SummaryMode::Count | SummaryMode::False) => {
            return Ok(resource);
        }
    };

    resource
        .filter(fields)
        .map_err(|e| OperationOutcomeError::error(IssueType::invalid(), e.to_string()))
}

fn filter_bundle(
    mut bundle: Bundle,
    subsetting: &Subsetting,
) -> Result<Bundle, OperationOutcomeError> {
    if matches!(subsetting, Subsetting::Summary(SummaryMode::Count)) {
        bundle.entry = None;
        return Ok(bundle);
    }

    let Some(entries) = bundle.entry.take() else {
        return Ok(bundle);
    };

    let filtered_entries = entries
        .into_iter()
        .map(|mut entry| {
            if let Some(resource) = entry.resource.take() {
                entry.resource = Some(Box::new(filter_resource(*resource, subsetting)?));
            }
            Ok(entry)
        })
        .collect::<Result<Vec<_>, OperationOutcomeError>>()?;

    bundle.entry = Some(filtered_entries);
    Ok(bundle)
}

fn filter_response(
    response: FHIRResponse,
    subsetting: &Subsetting,
) -> Result<FHIRResponse, OperationOutcomeError> {
    match response {
        FHIRResponse::Search(SearchResponse::Type(mut type_response)) => {
            type_response.bundle = filter_bundle(type_response.bundle, subsetting)?;
            Ok(FHIRResponse::Search(SearchResponse::Type(type_response)))
        }
        FHIRResponse::Search(SearchResponse::System(mut system_response)) => {
            system_response.bundle = filter_bundle(system_response.bundle, subsetting)?;
            Ok(FHIRResponse::Search(SearchResponse::System(
                system_response,
            )))
        }
        other => Ok(other),
    }
}

impl<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>
    MiddlewareChain<
        ServerMiddlewareState<Repo, Search, Terminology>,
        Arc<ServerCTX<Client>>,
        FHIRRequest,
        FHIRResponse,
        OperationOutcomeError,
    > for Middleware
{
    fn call(
        &self,
        state: ServerMiddlewareState<Repo, Search, Terminology>,
        context: ServerMiddlewareContext<Client>,
        next: Option<
            Arc<ServerMiddlewareNext<Client, ServerMiddlewareState<Repo, Search, Terminology>>>,
        >,
    ) -> ServerMiddlewareOutput<Client> {
        Box::pin(async move {
            let mut context = if let Some(next) = next {
                next(state, context).await
            } else {
                Err(OperationOutcomeError::fatal(
                    IssueType::exception(),
                    "No next middleware found".to_string(),
                ))
            }?;

            let Some(elements_summary_parameter) = get_summary_element_parameter(&context.request)
            else {
                return Ok(context);
            };

            let Some(subsetting) = subsetting(&elements_summary_parameter) else {
                return Ok(context);
            };

            if let Some(response) = context.response.take() {
                context.response = Some(filter_response(response, &subsetting)?);
            }

            Ok(context)
        })
    }
}
