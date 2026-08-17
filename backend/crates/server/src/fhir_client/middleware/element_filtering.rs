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

fn filter_resource(resource: Resource, fields: &[&str]) -> Result<Resource, OperationOutcomeError> {
    resource
        .filter(fields)
        .map_err(|e| OperationOutcomeError::error(IssueType::invalid(), e.to_string()))
}

fn filter_bundle(mut bundle: Bundle, fields: &[&str]) -> Result<Bundle, OperationOutcomeError> {
    let Some(entries) = bundle.entry.take() else {
        return Ok(bundle);
    };

    let filtered_entries = entries
        .into_iter()
        .map(|mut entry| {
            if let Some(resource) = entry.resource.take() {
                entry.resource = Some(Box::new(filter_resource(*resource, fields)?));
            }
            Ok(entry)
        })
        .collect::<Result<Vec<_>, OperationOutcomeError>>()?;

    bundle.entry = Some(filtered_entries);
    Ok(bundle)
}

fn filter_response(
    response: FHIRResponse,
    fields: &[&str],
) -> Result<FHIRResponse, OperationOutcomeError> {
    match response {
        FHIRResponse::Search(SearchResponse::Type(mut type_response)) => {
            type_response.bundle = filter_bundle(type_response.bundle, fields)?;
            Ok(FHIRResponse::Search(SearchResponse::Type(type_response)))
        }
        FHIRResponse::Search(SearchResponse::System(mut system_response)) => {
            system_response.bundle = filter_bundle(system_response.bundle, fields)?;
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

            let fields = requested_elements(&elements_summary_parameter);
            if fields.is_empty() {
                return Ok(context);
            }

            if let Some(response) = context.response.take() {
                context.response = Some(filter_response(response, &fields)?);
            }

            Ok(context)
        })
    }
}
