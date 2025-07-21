use crate::ServerErrors;
use fhir_model::r4::types::Resource;
pub mod postgres;

pub struct UserId(String);
impl UserId {
    pub fn new(id: String) -> Self {
        UserId(id)
    }
}
pub struct TenantId(String);
impl TenantId {
    pub fn new(id: String) -> Self {
        TenantId(id)
    }
}
pub struct ProjectId(String);
impl ProjectId {
    pub fn new(id: String) -> Self {
        ProjectId(id)
    }
}
pub struct VersionId(String);
impl VersionId {
    pub fn new(id: String) -> Self {
        VersionId(id)
    }
}

pub struct ResourceId(String);
impl ResourceId {
    pub fn new(id: String) -> Self {
        ResourceId(id)
    }
}

pub trait FHIRRepository: Send + Sync {
    fn insert(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        user_id: UserId,
        resource: Resource,
    ) -> impl Future<Output = Result<Resource, ServerErrors>> + Send;
    fn read_by_version_id(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        version_id: Vec<VersionId>,
    ) -> impl Future<Output = Result<Vec<Resource>, ServerErrors>> + Send;
    fn read_latest(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        resource_id: ResourceId,
    ) -> impl Future<Output = Result<Option<Resource>, ServerErrors>> + Send;
    fn history(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        resource_id: ResourceId,
    ) -> impl Future<Output = Result<Vec<Resource>, ServerErrors>> + Send;
    fn get_sequence(
        &self,
        tenant_id: TenantId,
        project_id: ProjectId,
        sequence_id: u64,
        count: Option<u64>,
    ) -> impl Future<Output = Result<Vec<Resource>, ServerErrors>> + Send;
}
