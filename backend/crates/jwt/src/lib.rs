use derivative::Derivative;
use haste_fhir_model::r4::generated::terminology::{BoundCode, UserRole as FHIRUserRole};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[cfg(feature = "reflect")]
pub mod reflect;

#[cfg(feature = "sqlx")]
pub mod sqlx;

pub mod claims;
pub mod scopes;

// Reserved keyword for system tenant, author and project.
static SYSTEM: &str = "system";

#[derive(Clone, Debug, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Owner,
    Admin,
    Member,
}

impl From<BoundCode<FHIRUserRole>> for UserRole {
    fn from(role: BoundCode<FHIRUserRole>) -> Self {
        if role == FHIRUserRole::owner() {
            UserRole::Owner
        } else if role == FHIRUserRole::admin() {
            UserRole::Admin
        } else {
            UserRole::Member
        }
    }
}

#[derive(Derivative, Clone)]
#[derivative(Debug = "transparent")]
pub enum AuthorId {
    // System is used for system level actions such as tenant creation etc..
    System,
    User(ResourceId),
}

impl AuthorId {
    #[must_use]
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
            AuthorId::System => write!(f, "{SYSTEM}"),
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

#[derive(Derivative, Clone, PartialEq, Eq, Hash)]
#[derivative(Debug = "transparent")]
pub enum TenantId {
    System,
    #[derivative(Debug = "transparent")]
    Custom(String),
}

impl TenantId {
    #[must_use]
    pub fn new(id: String) -> Self {
        // Should never be able to create a system tenant from user.
        if id == SYSTEM {
            TenantId::System
        } else {
            TenantId::Custom(id)
        }
    }
}

impl From<String> for TenantId {
    fn from(id: String) -> Self {
        TenantId::new(id)
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
            TenantId::System => write!(f, "{SYSTEM}"),
            TenantId::Custom(id) => write!(f, "{id}"),
        }
    }
}

#[derive(Derivative, Clone, PartialEq, Eq, Hash)]
#[derivative(Debug = "transparent")]
pub enum ProjectId {
    System,
    #[derivative(Debug = "transparent")]
    Custom(String),
}
impl ProjectId {
    #[must_use]
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
            ProjectId::System => write!(f, "{SYSTEM}"),
            ProjectId::Custom(id) => write!(f, "{id}"),
        }
    }
}

pub struct VersionIdRef<'a>(&'a str);
impl<'a> VersionIdRef<'a> {
    #[must_use]
    pub fn new(id: &'a str) -> Self {
        VersionIdRef(id)
    }
}
impl<'a> AsRef<str> for VersionIdRef<'a> {
    fn as_ref(&self) -> &'a str {
        self.0
    }
}
impl<'a> From<&'a VersionId> for VersionIdRef<'a> {
    fn from(version_id: &'a VersionId) -> Self {
        VersionIdRef::new(&version_id.0)
    }
}

#[derive(Derivative, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
#[derivative(Debug = "transparent")]
pub struct VersionId(String);
impl VersionId {
    #[must_use]
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

#[derive(Derivative, Clone, Deserialize, Serialize)]
#[derivative(Debug = "transparent")]
pub struct ResourceId(String);
impl ResourceId {
    #[must_use]
    pub fn new(id: String) -> Self {
        ResourceId(id)
    }
}
impl AsRef<str> for ResourceId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
