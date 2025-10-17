use oxidized_fhir_client::request::FHIRRequest;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};
mod sqlx_bindings;

pub mod authorization_code;
pub mod membership;
pub mod project;
pub mod tenant;
pub mod user;

// Reserved keyword for system tenant, author and project.
static SYSTEM: &str = "system";

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, serde::Deserialize, serde::Serialize)]
#[sqlx(type_name = "fhir_version", rename_all = "lowercase")] // only for PostgreSQL to match a type definition
#[serde(rename_all = "lowercase")]
pub enum SupportedFHIRVersions {
    R4,
}

#[derive(Clone, Debug)]
pub enum AuthorId {
    // System is used for system level actions such as tenant creation etc..
    System,
    User(ResourceId),
}
impl AuthorId {
    pub fn new(id: String) -> Self {
        // Should never be able to create a system author from user.
        if id == SYSTEM {
            AuthorId::System
        } else {
            AuthorId::User(ResourceId::new(id))
        }
    }
}
impl AsRef<str> for AuthorId {
    fn as_ref(&self) -> &str {
        match self {
            AuthorId::System => SYSTEM,
            AuthorId::User(id) => id.as_ref(),
        }
    }
}

impl<'de> Deserialize<'de> for AuthorId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(AuthorId::new(String::deserialize(deserializer)?))
    }
}

impl Serialize for AuthorId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_ref())
    }
}

impl Display for AuthorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthorId::System => write!(f, "{}", SYSTEM),
            AuthorId::User(id) => write!(f, "{}", id.as_ref()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum AuthorKind {
    System,
    Membership,
    ClientApplication,
    OperationDefinition,
}

impl AsRef<str> for AuthorKind {
    fn as_ref(&self) -> &str {
        match self {
            AuthorKind::System => "System",
            AuthorKind::Membership => "Membership",
            AuthorKind::ClientApplication => "ClientApplication",
            AuthorKind::OperationDefinition => "OperationDefinition",
        }
    }
}

impl Display for AuthorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

#[derive(Clone, Debug)]
pub struct Author {
    pub id: AuthorId,
    pub kind: AuthorKind,
}

impl std::fmt::Display for SupportedFHIRVersions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SupportedFHIRVersions::R4 => write!(f, "r4"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TenantId {
    System,
    Custom(String),
}

impl TenantId {
    pub fn new(id: String) -> Self {
        // Should never be able to create a system tenant from user.
        if id == SYSTEM {
            TenantId::System
        } else {
            TenantId::Custom(id)
        }
    }
}

impl AsRef<str> for TenantId {
    fn as_ref(&self) -> &str {
        match self {
            TenantId::System => SYSTEM,
            TenantId::Custom(id) => id,
        }
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

impl Serialize for TenantId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_ref())
    }
}

impl Display for TenantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TenantId::System => write!(f, "{}", SYSTEM),
            TenantId::Custom(id) => write!(f, "{}", id),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProjectId {
    System,
    Custom(String),
}
impl ProjectId {
    pub fn new(id: String) -> Self {
        // Should never be able to create a system project from user.
        if id == SYSTEM {
            ProjectId::System
        } else {
            ProjectId::Custom(id)
        }
    }
}

impl AsRef<str> for ProjectId {
    fn as_ref(&self) -> &str {
        match self {
            ProjectId::System => SYSTEM,
            ProjectId::Custom(id) => id,
        }
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

impl Serialize for ProjectId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_ref())
    }
}

impl Display for ProjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectId::System => write!(f, "{}", SYSTEM),
            ProjectId::Custom(id) => write!(f, "{}", id),
        }
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
impl<'a> From<&'a VersionId> for VersionIdRef<'a> {
    fn from(version_id: &'a VersionId) -> Self {
        VersionIdRef::new(&version_id.0)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VersionId(String);
impl VersionId {
    pub fn new(id: String) -> Self {
        VersionId(id)
    }
}
impl From<String> for VersionId {
    fn from(id: String) -> Self {
        VersionId(id)
    }
}
impl AsRef<str> for VersionId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
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
