use crate::fhir_client::{
    ClientState, ServerCTX,
    middleware::{Backend, Middleware, Next, Output},
    utilities::is_search_match,
};
use haste_fhir_client::{
    request::{FHIRRequest, FHIRResponse, SearchRequest, SearchResponse},
    url::{ParsedParameter, ParsedParameters},
};
use haste_fhir_model::r4::generated::{resources::Bundle, terminology::IssueType};
use haste_fhir_operation_error::OperationOutcomeError;
use std::sync::Arc;

/// Element Filtering (should occur after access control checks).
pub struct ElementFiltering;

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
fn requested_elements(parameters: &[&ParsedParameter]) -> Vec<String> {
    parameters
        .iter()
        .filter_map(|p| match p {
            ParsedParameter::Result(param) if param.name == "_elements" => Some(param),
            _ => None,
        })
        .flat_map(|param| param.value.iter().cloned())
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

/// What the caller asked to keep of each match. Owned, since it is decided
/// from the request and applied to the response once the request is gone.
enum Subsetting {
    Elements(Vec<String>),
    Summary(SummaryMode),
}

/// `_elements` takes precedence over `_summary` when both are present; the
/// spec doesn't define combined semantics for using both at once.
fn subsetting(parameters: &[&ParsedParameter]) -> Option<Subsetting> {
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

/// The top-level elements each match is cut down to, or `None` when matches
/// pass through whole.
fn kept_fields(subsetting: &Subsetting) -> Option<Vec<&str>> {
    match subsetting {
        Subsetting::Elements(fields) => Some(fields.iter().map(String::as_str).collect()),
        // Do not have access yet to fields outside of minimum required yet.
        // Will need to mark fields in generated structs with #[summary] or something else.
        Subsetting::Summary(SummaryMode::True) => Some(Vec::new()),
        Subsetting::Summary(SummaryMode::Text) => Some(vec!["text"]),
        // Count is handled at the bundle level (no entries at all); Data
        // isn't implemented yet. Both pass the resource through unchanged.
        Subsetting::Summary(SummaryMode::Data | SummaryMode::Count | SummaryMode::False) => None,
    }
}

fn filter_bundle(
    mut bundle: Bundle,
    subsetting: &Subsetting,
) -> Result<Bundle, OperationOutcomeError> {
    if matches!(subsetting, Subsetting::Summary(SummaryMode::Count)) {
        bundle.entry = None;
        return Ok(bundle);
    }

    let Some(fields) = kept_fields(subsetting) else {
        return Ok(bundle);
    };
    let Some(entries) = bundle.entry.take() else {
        return Ok(bundle);
    };

    let filtered_entries = entries
        .into_iter()
        .map(|mut entry| {
            if is_search_match(&entry)
                && let Some(resource) = entry.resource.take()
            {
                let filtered = resource.filter(&fields).map_err(|e| {
                    OperationOutcomeError::error(IssueType::invalid(), e.to_string())
                })?;
                entry.resource = Some(Box::new(filtered));
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

impl<B: Backend> Middleware<B> for ElementFiltering {
    fn call<'a>(
        &'a self,
        state: &'a ClientState<B>,
        ctx: Arc<ServerCTX<B>>,
        request: FHIRRequest,
        next: Next<'a, B>,
    ) -> Output<'a> {
        // Decided before the request moves on: the response alone does not
        // say what was asked for.
        let Some(subsetting) =
            get_summary_element_parameter(&request).and_then(|parameters| subsetting(&parameters))
        else {
            return next.run(state, ctx, request);
        };

        Box::pin(async move {
            let response = next.run(state, ctx, request).await?;
            filter_response(response, &subsetting)
        })
    }
}
