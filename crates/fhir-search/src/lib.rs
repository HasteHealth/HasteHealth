use oxidized_fhir_client::request::{FHIRSearchSystemRequest, FHIRSearchTypeRequest};
use oxidized_fhir_model::r4::types::Resource;
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_repository::{ProjectId, SupportedFHIRVersions, TenantId};

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
    ) -> impl Future<Output = Result<Vec<String>, OperationOutcomeError>> + Send;

    fn index(
        &self,
        tenant: TenantId,
        project: ProjectId,
        resource: Vec<Resource>,
    ) -> impl Future<Output = Result<(), OperationOutcomeError>> + Send;

    fn remove_index(
        &self,
        tenant: TenantId,
        project: ProjectId,
        remove_indices: Vec<RemoveIndex>,
    ) -> impl Future<Output = Result<(), OperationOutcomeError>> + Send;

    fn migrate(
        &self,
        fhir_version: SupportedFHIRVersions,
        index: &str,
    ) -> impl Future<Output = Result<(), oxidized_fhir_operation_error::OperationOutcomeError>> + Send;
}
