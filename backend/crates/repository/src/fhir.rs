/// FHIR Access
use crate::types::{FHIRMethod, SupportedFHIRVersions};
use oxidized_fhir_client::request::{
    FHIRHistoryInstanceRequest, FHIRHistorySystemRequest, FHIRHistoryTypeRequest,
};
use oxidized_fhir_model::r4::generated::resources::{Resource, ResourceType};
use oxidized_fhir_model::r4::sqlx::FHIRJson;
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_jwt::{ProjectId, ResourceId, TenantId, VersionId, claims::UserTokenClaims};

pub struct ResourcePollingValue {
    pub id: ResourceId,
    pub resource_type: ResourceType,
    pub version_id: String,
    pub project: ProjectId,
    pub tenant: TenantId,
    pub resource: FHIRJson<Resource>,
    pub sequence: i64,
    pub fhir_method: FHIRMethod,
}

pub enum HistoryRequest<'a> {
    System(&'a FHIRHistorySystemRequest),
    Type(&'a FHIRHistoryTypeRequest),
    Instance(&'a FHIRHistoryInstanceRequest),
}

#[derive(PartialEq, Eq)]
pub enum CachePolicy {
    NoCache,
    Cache,
}

pub trait FHIRRepository: Sized {
    fn create(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        user: &UserTokenClaims,
        fhir_version: &SupportedFHIRVersions,
        resource: &mut Resource,
    ) -> impl Future<Output = Result<Resource, OperationOutcomeError>> + Send;

    fn update(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        user: &UserTokenClaims,
        fhir_version: &SupportedFHIRVersions,
        resource: &mut Resource,
        id: &str,
    ) -> impl Future<Output = Result<Resource, OperationOutcomeError>> + Send;

    fn delete(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        user: &UserTokenClaims,
        fhir_version: &SupportedFHIRVersions,
        resource: &mut Resource,
        id: &str,
    ) -> impl Future<Output = Result<Resource, OperationOutcomeError>> + Send;

    fn read_by_version_ids(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        version_id: &[&VersionId],
        cache_policy: CachePolicy,
    ) -> impl Future<Output = Result<Vec<Resource>, OperationOutcomeError>> + Send;
    fn read_latest(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        resource_type: &ResourceType,
        resource_id: &ResourceId,
    ) -> impl Future<Output = Result<Option<Resource>, OperationOutcomeError>> + Send;
    fn history(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        request: HistoryRequest,
    ) -> impl Future<Output = Result<Vec<Resource>, OperationOutcomeError>> + Send;
    fn get_sequence(
        &self,
        tenant_id: &TenantId,
        sequence_id: u64,
        count: Option<u64>,
    ) -> impl Future<Output = Result<Vec<ResourcePollingValue>, OperationOutcomeError>> + Send;
    fn transaction<'a>(
        &'a self,
    ) -> impl Future<Output = Result<Self, OperationOutcomeError>> + Send;
    fn in_transaction(&self) -> bool;
    fn commit(self) -> impl Future<Output = Result<(), OperationOutcomeError>> + Send;
    fn rollback(self) -> impl Future<Output = Result<(), OperationOutcomeError>> + Send;
}
