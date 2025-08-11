use oxidized_fhir_client::request::FHIRRequest;
use oxidized_fhir_client::request::{
    FHIRHistoryInstanceRequest, FHIRHistorySystemRequest, FHIRHistoryTypeRequest,
};
use oxidized_fhir_model::r4::sqlx::FHIRJson;
use oxidized_fhir_model::r4::types::{Resource, ResourceType};
use oxidized_fhir_operation_error::OperationOutcomeError;
use serde::Deserialize;
use std::fmt::{Debug, Display};

pub mod postgres;
mod sqlx_bindings;
pub mod utilities;

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, serde::Deserialize, serde::Serialize)]
#[sqlx(type_name = "fhir_version", rename_all = "lowercase")] // only for PostgreSQL to match a type definition
pub enum SupportedFHIRVersions {
    R4,
    R4B,
    R5,
}

pub struct Author {
    pub id: String,
    pub kind: String,
}

impl std::fmt::Display for SupportedFHIRVersions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SupportedFHIRVersions::R4 => write!(f, "R4"),
            SupportedFHIRVersions::R4B => write!(f, "R4B"),
            SupportedFHIRVersions::R5 => write!(f, "R5"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TenantId(String);
impl TenantId {
    pub fn new(id: String) -> Self {
        TenantId(id)
    }
}

impl AsRef<str> for TenantId {
    fn as_ref(&self) -> &str {
        &self.0
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
#[derive(Debug, Clone)]
pub struct ProjectId(String);
impl ProjectId {
    pub fn new(id: String) -> Self {
        ProjectId(id)
    }
}
impl AsRef<str> for ProjectId {
    fn as_ref(&self) -> &str {
        &self.0
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
pub struct VersionIdRef<'a>(&'a str);
impl<'a> VersionIdRef<'a> {
    pub fn new(id: &'a str) -> Self {
        VersionIdRef(id)
    }
}
impl<'a> AsRef<str> for VersionIdRef<'a> {
    fn as_ref(&self) -> &'a str {
        &self.0
    }
}

#[derive(Clone)]
pub struct ResourceId(String);
impl ResourceId {
    pub fn new(id: String) -> Self {
        ResourceId(id)
    }
}
impl AsRef<str> for ResourceId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(sqlx::Type, Debug, Clone)]
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
pub struct InsertResourceRow<'a> {
    pub tenant: TenantId,
    pub project: ProjectId,

    pub author_id: String,
    pub resource: &'a Resource,
    pub deleted: bool,

    pub request_method: String,

    pub fhir_version: SupportedFHIRVersions,
    pub author_type: String,

    pub fhir_method: FHIRMethod,
}

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

pub trait FHIRRepository {
    type Transaction;

    fn create(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        author: &Author,
        fhir_version: &SupportedFHIRVersions,
        resource: &mut Resource,
    ) -> impl Future<Output = Result<Resource, OperationOutcomeError>> + Send;

    fn update(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        author: &Author,
        fhir_version: &SupportedFHIRVersions,
        resource: &mut Resource,
        id: &str,
    ) -> impl Future<Output = Result<Resource, OperationOutcomeError>> + Send;

    fn read_by_version_ids(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        version_id: Vec<VersionIdRef>,
    ) -> impl Future<Output = Result<Vec<Resource>, OperationOutcomeError>> + Send;
    fn read_latest(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        resource_type: &ResourceType,
        resource_id: &ResourceId,
    ) -> impl Future<
        Output = Result<Option<oxidized_fhir_model::r4::types::Resource>, OperationOutcomeError>,
    > + Send;
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

    fn transaction<'a>(&'a self) -> impl Future<Output = Option<Self::Transaction>> + Send;
}

pub trait FHIRTransaction<Connection> {
    fn create(
        k: Connection,
        tenant: &TenantId,
        project: &ProjectId,
        author: &Author,
        fhir_version: &SupportedFHIRVersions,
        resource: &mut Resource,
    ) -> impl Future<Output = Result<Resource, OperationOutcomeError>> + Send;

    fn update(
        k: Connection,
        tenant: &TenantId,
        project: &ProjectId,
        author: &Author,
        fhir_version: &SupportedFHIRVersions,
        resource: &mut Resource,
        id: &str,
    ) -> impl Future<Output = Result<Resource, OperationOutcomeError>> + Send;

    fn read_by_version_ids(
        k: Connection,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        version_id: Vec<VersionIdRef>,
    ) -> impl Future<Output = Result<Vec<Resource>, OperationOutcomeError>> + Send;
    fn read_latest(
        k: Connection,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        resource_type: &ResourceType,
        resource_id: &ResourceId,
    ) -> impl Future<
        Output = Result<Option<oxidized_fhir_model::r4::types::Resource>, OperationOutcomeError>,
    > + Send;
    fn history(
        k: Connection,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        request: HistoryRequest,
    ) -> impl Future<Output = Result<Vec<Resource>, OperationOutcomeError>> + Send;
    fn get_sequence(
        k: Connection,
        tenant_id: &TenantId,
        sequence_id: u64,
        count: Option<u64>,
    ) -> impl Future<Output = Result<Vec<ResourcePollingValue>, OperationOutcomeError>> + Send;
}
