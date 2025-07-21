use std::fmt::Display;

use crate::ServerErrors;
use fhir_model::r4::types::Resource;
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
