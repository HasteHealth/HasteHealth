use oxidized_fhir_client::request::{FHIRSearchSystemRequest, FHIRSearchTypeRequest};
use oxidized_fhir_model::r4::types::{Resource, ResourceType};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_repository::{
    FHIRMethod, ProjectId, ResourceId, SupportedFHIRVersions, TenantId,
};

pub mod elastic_search;
mod indexing_conversion;

pub enum SearchRequest<'a> {
    TypeSearch(&'a FHIRSearchTypeRequest),
    SystemSearch(&'a FHIRSearchSystemRequest),
}

pub struct RemoveIndex {
    // resource_type: ResourceType,
    // id: String,
}

pub struct IndexResource<'a> {
    pub id: &'a ResourceId,
    pub version_id: &'a String,

    pub project: &'a ProjectId,

    pub fhir_method: &'a FHIRMethod,

    pub resource_type: &'a ResourceType,
    pub resource: &'a Resource,
}

pub struct SearchReturn {
    pub total: Option<i64>,
    pub version_ids: Vec<String>,
}

pub trait SearchEngine: Send + Sync {
    fn search(
        &self,
        fhir_version: &SupportedFHIRVersions,
        tenant: &TenantId,
        project: &ProjectId,
        search_request: SearchRequest,
    ) -> impl Future<Output = Result<SearchReturn, OperationOutcomeError>> + Send;

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
