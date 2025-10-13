use oxidized_fhir_client::request::FHIRRequest;
use oxidized_fhir_model::r4::generated::resources::ResourceType;

pub fn request_to_resource_type<'a>(request: &'a FHIRRequest) -> Option<&'a ResourceType> {
    match request {
        FHIRRequest::Read(req) => Some(&req.resource_type),
        FHIRRequest::VersionRead(req) => Some(&req.resource_type),
        FHIRRequest::UpdateInstance(req) => Some(&req.resource_type),
        FHIRRequest::DeleteInstance(req) => Some(&req.resource_type),
        FHIRRequest::Patch(req) => Some(&req.resource_type),
        FHIRRequest::HistoryInstance(req) => Some(&req.resource_type),

        // Type operations
        FHIRRequest::Create(req) => Some(&req.resource_type),
        FHIRRequest::HistoryType(req) => Some(&req.resource_type),
        FHIRRequest::SearchType(req) => Some(&req.resource_type),
        FHIRRequest::ConditionalUpdate(req) => Some(&req.resource_type),
        FHIRRequest::DeleteType(req) => Some(&req.resource_type),
        _ => None,
    }
}
