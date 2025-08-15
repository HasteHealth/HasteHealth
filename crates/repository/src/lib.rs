use oxidized_fhir_client::request::FHIRRequest;
use serde::Deserialize;
use std::fmt::{Debug, Display};

pub mod auth;
pub mod fhir;
pub mod pg;
mod sqlx_bindings;
pub mod utilities;

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, serde::Deserialize, serde::Serialize)]
#[sqlx(type_name = "fhir_version", rename_all = "lowercase")] // only for PostgreSQL to match a type definition
pub enum SupportedFHIRVersions {
    R4,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, serde::Deserialize, serde::Serialize)]
#[sqlx(type_name = "auth_method", rename_all = "lowercase")] // only for PostgreSQL to match a type definition
pub enum AuthMethod {
    #[sqlx(rename = "email-password")]
    EmailPassword,
    OIDC,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, serde::Deserialize, serde::Serialize)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")] // only for PostgreSQL to match a type definition
pub enum UserRole {
    Owner,
    Admin,
    Member,
}

pub struct Author {
    pub id: String,
    pub kind: String,
}

impl std::fmt::Display for SupportedFHIRVersions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SupportedFHIRVersions::R4 => write!(f, "R4"),
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
