use haste_fhir_client::request::{
    DeleteRequest, FHIRRequest, HistoryRequest, InvocationRequest, SearchRequest, UpdateRequest,
};
use haste_fhir_model::r4::generated::resources::ResourceType;

/// Converts a FHIRRequest to its corresponding ResourceType if applicable.
pub fn request_to_resource_type<'a>(request: &'a FHIRRequest) -> Option<&'a ResourceType> {
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
