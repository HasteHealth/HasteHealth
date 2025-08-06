use oxidized_fhir_client::request::{FHIRSearchSystemRequest, FHIRSearchTypeRequest};
use oxidized_fhir_model::r4::types::{Resource, ResourceType};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_repository::{FHIRMethod, ProjectId, SupportedFHIRVersions, TenantId, VersionId};

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
    id: String,
    version_id: VersionId<'a>,
    tenant: TenantId,
    project: ProjectId,
    fhir_method: FHIRMethod,

    resource_type: ResourceType,
    resource: &'a Resource,
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
        tenant: TenantId,
        project: ProjectId,
        resource: Vec<IndexResource>,
    ) -> impl Future<Output = Result<(), OperationOutcomeError>> + Send;

    fn migrate(
        &self,
        fhir_version: &SupportedFHIRVersions,
    ) -> impl Future<Output = Result<(), oxidized_fhir_operation_error::OperationOutcomeError>> + Send;
}
