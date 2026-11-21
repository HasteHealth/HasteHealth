use haste_fhir_client::request::{
    DeleteRequest, FHIRRequest, FHIRResponse, HistoryRequest, InvocationRequest, SearchRequest,
    SearchResponse, UpdateRequest,
};
use haste_fhir_model::r4::generated::{
    resources::{BundleEntry, ResourceType},
    terminology::SearchEntryMode,
};

/// Converts a FHIRRequest to its corresponding ResourceType if applicable.
pub fn request_to_resource_type(request: &FHIRRequest) -> Option<&ResourceType> {
    match request {
        FHIRRequest::Create(req) => Some(&req.resource_type),
        // Instance Operations
        FHIRRequest::Read(req) => Some(&req.resource_type),
        FHIRRequest::VersionRead(req) => Some(&req.resource_type),

        FHIRRequest::Patch(req) => Some(&req.resource_type),

        FHIRRequest::Update(UpdateRequest::Instance(req)) => Some(&req.resource_type),
        FHIRRequest::Update(UpdateRequest::Conditional(req)) => Some(&req.resource_type),

        FHIRRequest::History(HistoryRequest::Instance(req)) => Some(&req.resource_type),
        FHIRRequest::History(HistoryRequest::Type(req)) => Some(&req.resource_type),

        FHIRRequest::Delete(DeleteRequest::Instance(req)) => Some(&req.resource_type),
        FHIRRequest::Delete(DeleteRequest::Type(req)) => Some(&req.resource_type),

        FHIRRequest::Invocation(InvocationRequest::Instance(req)) => Some(&req.resource_type),
        FHIRRequest::Invocation(InvocationRequest::Type(req)) => Some(&req.resource_type),

        FHIRRequest::Search(SearchRequest::Type(req)) => Some(&req.resource_type),

        FHIRRequest::Compartment(compartment_request) => {
            request_to_resource_type(&compartment_request.request)
        }

        // System operations
        FHIRRequest::History(HistoryRequest::System(_))
        | FHIRRequest::Delete(DeleteRequest::System(_))
        | FHIRRequest::Capabilities
        | FHIRRequest::Search(SearchRequest::System(_))
        | &FHIRRequest::Invocation(InvocationRequest::System(_))
        | FHIRRequest::Batch(_)
        | FHIRRequest::Transaction(_) => None,
    }
}

/// Whether `entry` is a search match rather than something `_include` or
/// `_revinclude` added beside it. An entry with no `search.mode` (a level that
/// doesn't yet stamp one, such as history or system search) is treated as a
/// match, since every entry there is one.
///
/// Access control and `_elements`/`_summary` must both apply only to what the
/// client actually searched for: an include exists to give context for a
/// match, not to smuggle in a resource the request itself wouldn't return.
pub fn is_search_match(entry: &BundleEntry) -> bool {
    entry
        .search
        .as_ref()
        .and_then(|search| search.mode.as_ref())
        .is_none_or(|mode| mode == &SearchEntryMode::match_())
}

/// Applies `f` to a search response's bundle entries; other responses pass
/// through unchanged.
pub async fn map_search_entries<F, Fut>(response: FHIRResponse, f: F) -> FHIRResponse
where
    F: FnOnce(Vec<BundleEntry>) -> Fut,
    Fut: Future<Output = Vec<BundleEntry>>,
{
    match response {
        FHIRResponse::Search(SearchResponse::Type(mut r)) => {
            if let Some(entries) = r.bundle.entry.take() {
                r.bundle.entry = Some(f(entries).await);
            }
            FHIRResponse::Search(SearchResponse::Type(r))
        }
        FHIRResponse::Search(SearchResponse::System(mut r)) => {
            if let Some(entries) = r.bundle.entry.take() {
                r.bundle.entry = Some(f(entries).await);
            }
            FHIRResponse::Search(SearchResponse::System(r))
        }
        other => other,
    }
}
