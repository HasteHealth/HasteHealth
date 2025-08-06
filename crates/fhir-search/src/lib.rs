use oxidized_fhir_client::request::{FHIRSearchSystemRequest, FHIRSearchTypeRequest};
use oxidized_fhir_model::r4::types::{Resource, ResourceType};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_repository::{
    FHIRMethod, ProjectId, ResourceId, SupportedFHIRVersions, TenantId, VersionId,
};

pub mod elastic_search;
mod indexing_conversion;

pub enum SearchRequest {
    TypeSearch(FHIRSearchTypeRequest),
    SystemSearch(FHIRSearchSystemRequest),
}

pub struct RemoveIndex {
    // resource_type: ResourceType,
    // id: String,
}

pub struct IndexResource<'a> {
    pub id: ResourceId,
    pub version_id: String,

    pub project: ProjectId,

    pub fhir_method: FHIRMethod,

    pub resource_type: ResourceType,
    pub resource: &'a Resource,
}

pub trait SearchEngine {
    fn search(
        &self,
        fhir_version: &SupportedFHIRVersions,
        tenant: TenantId,
        project: ProjectId,
        search_request: SearchRequest,
    ) -> impl Future<Output = Result<Vec<String>, OperationOutcomeError>> + Send;

    fn index(
        &self,
        fhir_version: &SupportedFHIRVersions,
        tenant: &TenantId,
        resource: Vec<IndexResource>,
    ) -> impl Future<Output = Result<(), OperationOutcomeError>> + Send;

    fn migrate(
        &self,
        fhir_version: &SupportedFHIRVersions,
    ) -> impl Future<Output = Result<(), oxidized_fhir_operation_error::OperationOutcomeError>> + Send;
}
