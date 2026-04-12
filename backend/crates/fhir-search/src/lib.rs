use haste_fhir_client::request::SearchRequest;
use haste_fhir_model::r4::generated::resources::{Resource, ResourceType};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, ResourceId, TenantId, VersionId};
use haste_repository::types::{FHIRMethod, SupportedFHIRVersions};
use serde::Deserialize;

pub mod elastic_search;
pub mod indexing_conversion;

#[derive(Clone)]
pub struct IndexResource {
    pub id: ResourceId,
    pub version_id: VersionId,

    pub tenant: TenantId,
    pub project: ProjectId,

    pub fhir_method: FHIRMethod,
    pub resource_type: ResourceType,
    pub resource: Resource,
}

#[derive(Deserialize, Debug)]
pub struct SearchEntry {
    pub id: ResourceId,
    pub resource_type: ResourceType,
    pub version_id: VersionId,
    pub project: ProjectId,
}

pub struct SearchReturn {
    pub total: Option<i64>,
    pub entries: Vec<SearchEntry>,
}

pub struct SearchOptions {
    pub count_limit: bool,
}

pub struct SuccessfullyIndexedCount(pub usize);

pub trait SearchEngine: Send + Sync {
    fn search(
        &self,
        fhir_version: &SupportedFHIRVersions,
        tenant: &TenantId,
        projects: &[&ProjectId],
        search_request: &SearchRequest,
        options: Option<SearchOptions>,
    ) -> impl Future<Output = Result<SearchReturn, OperationOutcomeError>> + Send + Sync;

    fn index(
        &self,
        fhir_version: SupportedFHIRVersions,
        resource: Vec<IndexResource>,
    ) -> impl Future<Output = Result<SuccessfullyIndexedCount, OperationOutcomeError>> + Send + Sync;

    fn migrate(
        &self,
        fhir_version: &SupportedFHIRVersions,
    ) -> impl Future<Output = Result<(), haste_fhir_operation_error::OperationOutcomeError>> + Send + Sync;
}
