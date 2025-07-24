use std::fmt::Display;

use crate::{ServerErrors, SupportedFHIRVersions};
use fhir_client::request::FHIRRequest;
use fhir_model::r4::{sqlx::FHIRJson, types::Resource};
use serde::Deserialize;
pub mod postgres;

pub struct UserId(String);
impl UserId {
    pub fn new(id: String) -> Self {
        UserId(id)
    }
}

#[derive(Debug)]
pub struct TenantId(String);
impl TenantId {
    pub fn new(id: String) -> Self {
        TenantId(id)
    }
}
impl<'de> Deserialize<'de> for TenantId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(TenantId::new(String::deserialize(deserializer)?))
    }
}

impl Display for TenantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
#[derive(Debug)]
pub struct ProjectId(String);
impl ProjectId {
    pub fn new(id: String) -> Self {
        ProjectId(id)
    }
}
impl<'de> Deserialize<'de> for ProjectId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(ProjectId::new(String::deserialize(deserializer)?))
    }
}

impl Display for ProjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
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

#[derive(sqlx::Type)]
#[sqlx(type_name = "fhir_method", rename_all = "lowercase")]
pub enum FHIRMethod {
    Create,
    Read,
    Update,
    Delete,
}

impl TryFrom<&FHIRRequest> for FHIRMethod {
    type Error = String;

    fn try_from(request: &FHIRRequest) -> Result<Self, Self::Error> {
        match request {
            FHIRRequest::Create(_) => Ok(FHIRMethod::Create),
            FHIRRequest::Read(_) => Ok(FHIRMethod::Read),
            FHIRRequest::UpdateInstance(_) => Ok(FHIRMethod::Update),
            FHIRRequest::ConditionalUpdate(_) => Ok(FHIRMethod::Update),
            FHIRRequest::DeleteInstance(_) => Ok(FHIRMethod::Delete),
            FHIRRequest::DeleteType(_) => Ok(FHIRMethod::Delete),
            FHIRRequest::DeleteSystem(_) => Ok(FHIRMethod::Delete),
            _ => Err("Unsupported FHIR request".to_string()),
        }
    }
}

#[derive(sqlx::FromRow)]
pub struct InsertResourceRow {
    pub tenant: String,
    pub project: String,
    // resource_type: String,
    pub author_id: String,
    pub resource: FHIRJson<Resource>,
    pub deleted: bool,
    // created_at: chrono::DateTime<Utc>,
    pub request_method: String,

    pub fhir_version: SupportedFHIRVersions,
    pub author_type: String,
    // version_id: String,
    pub fhir_method: FHIRMethod,
    // sequence: i64,
}

pub trait FHIRRepository: Send + Sync {
    fn insert(
        &self,
        insertion: &InsertResourceRow,
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
