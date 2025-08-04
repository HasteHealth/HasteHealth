use crate::repository::{ProjectId, TenantId, VersionId};
use oxidized_fhir_client::request::{FHIRSearchSystemRequest, FHIRSearchTypeRequest};
use oxidized_fhir_model::r4::types::{Resource, ResourceType};
use oxidized_fhir_operation_error::OperationOutcomeError;

mod elastic_search;

pub enum SearchRequest {
    TypeSearch(FHIRSearchTypeRequest),
    SystemSearch(FHIRSearchSystemRequest),
}

pub struct RemoveIndex {
    resource_type: ResourceType,
    id: String,
}

pub trait SearchEngine {
    fn search(
        tenant: TenantId,
        project: ProjectId,
        search_request: SearchRequest,
    ) -> Result<Vec<String>, OperationOutcomeError>;

    fn index(
        tenant: TenantId,
        project: ProjectId,
        resource: Vec<Resource>,
    ) -> Result<(), OperationOutcomeError>;

    fn remove_index(
        tenant: TenantId,
        project: ProjectId,
        remove_indices: Vec<RemoveIndex>,
    ) -> Result<(), OperationOutcomeError>;
}
