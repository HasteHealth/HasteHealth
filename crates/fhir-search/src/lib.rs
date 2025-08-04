use oxidized_fhir_client::request::{FHIRSearchSystemRequest, FHIRSearchTypeRequest};
use oxidized_fhir_model::r4::types::Resource;
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_repository::{ProjectId, TenantId};

pub mod elastic_search;

pub enum SearchRequest {
    TypeSearch(FHIRSearchTypeRequest),
    SystemSearch(FHIRSearchSystemRequest),
}

pub struct RemoveIndex {
    // resource_type: ResourceType,
    // id: String,
}

pub trait SearchEngine {
    fn search(
        &self,
        tenant: TenantId,
        project: ProjectId,
        search_request: SearchRequest,
    ) -> Result<Vec<String>, OperationOutcomeError>;

    fn index(
        &self,
        tenant: TenantId,
        project: ProjectId,
        resource: Vec<Resource>,
    ) -> Result<(), OperationOutcomeError>;

    fn remove_index(
        &self,
        tenant: TenantId,
        project: ProjectId,
        remove_indices: Vec<RemoveIndex>,
    ) -> Result<(), OperationOutcomeError>;

    fn migrate(&self) -> Result<(), oxidized_fhir_operation_error::OperationOutcomeError>;
}
