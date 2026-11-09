use derivative::Derivative;
use haste_fhir_model::r4::generated::terminology::{
    BoundCode, IssueType, UserRole as FHIRUserRole,
};
use haste_fhir_operation_error::OperationOutcomeError;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[cfg(feature = "reflect")]
pub mod reflect;

#[cfg(feature = "sqlx")]
pub mod sqlx;

pub mod claims;
pub mod scopes;

#[cfg(test)]
mod proptests;

// Reserved keyword for system tenant, author and project.
static SYSTEM: &str = "system";

/// What a tenant or project id looks like: lowercase ASCII letters and digits
/// joined by single hyphens, at most [`HOSTNAME_ID_MAX_LEN`] bytes. A
/// [`TenantId`] or [`ProjectId`] holds nothing else, so an id is safe as a URL
/// path segment, a subdomain label, a directory name or a storage key wherever
/// it goes. It never holds `--`, so the hostname label `{tenant}--{project}`
/// splits back apart.
///
/// The sign-up form uses it as an HTML `pattern`, so it must stay valid under
/// the browser's `v` regex flag (no unescaped `-` inside `[]`).
pub const HOSTNAME_ID_PATTERN: &str = "[a-z0-9]+(-[a-z0-9]+)*";

/// A DNS label holds 63 characters.
pub const HOSTNAME_ID_MAX_LEN: usize = 63;

/// Why `id` does not match [`HOSTNAME_ID_PATTERN`], if it does not. The text
/// completes a sentence such as "A tenant id …".
#[must_use]
pub fn hostname_id_problem(id: &str) -> Option<&'static str> {
    if id.is_empty() {
        Some("must not be empty")
    } else if id.len() > HOSTNAME_ID_MAX_LEN {
        Some("may be at most 63 characters long")
    } else if !id
        .bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        Some("may only contain lowercase letters, digits and hyphens")
    } else if id.starts_with('-') || id.ends_with('-') || id.contains("--") {
        Some("may only have single hyphens, between letters or digits")
    } else {
        None
    }
}

fn invalid_hostname_id(kind: &str, id: &str, problem: &str) -> OperationOutcomeError {
    OperationOutcomeError::error(
        IssueType::invalid(),
        format!("A {kind} id {problem}: '{id}'."),
    )
}

/// Lowercase letters and digits: a random string of them is a valid id at any
/// length from 1 to [`HOSTNAME_ID_MAX_LEN`].
static HOSTNAME_ID_ALPHABET: [char; 36] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
    'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];

/// A random id of `len` lowercase letters and digits. `len` is clamped to
/// 1 to [`HOSTNAME_ID_MAX_LEN`], so the result always matches
/// [`HOSTNAME_ID_PATTERN`].
#[must_use]
pub fn random_hostname_id(len: usize) -> String {
    let len = len.clamp(1, HOSTNAME_ID_MAX_LEN);
    nanoid::nanoid!(len, &HOSTNAME_ID_ALPHABET)
}

/// The FHIR versions a project can be served as.
///
/// A project's version is fixed when it is created and cannot be changed
/// afterwards, so it is safe to cache and to carry in an access token.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Deserialize, Serialize)]
#[cfg_attr(feature = "sqlx", derive(::sqlx::Type))]
#[cfg_attr(
    feature = "sqlx",
    sqlx(type_name = "fhir_version", rename_all = "lowercase")
)]
#[serde(rename_all = "lowercase")]
pub enum SupportedFHIRVersions {
    #[default]
    R4,
}

impl SupportedFHIRVersions {
    /// Every version a project can be served as, in the order routes are
    /// registered for them.
    pub const ALL: &[SupportedFHIRVersions] = &[SupportedFHIRVersions::R4];

    /// The version as it appears in a URL.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            SupportedFHIRVersions::R4 => "r4",
        }
    }

    /// The version a URL segment names, if it names one at all. Used to tell a
    /// leading version segment apart from a resource type.
    #[must_use]
    pub fn from_url_segment(segment: &str) -> Option<SupportedFHIRVersions> {
        SupportedFHIRVersions::ALL
            .iter()
            .find(|version| version.as_str() == segment)
            .cloned()
    }
}

impl Display for SupportedFHIRVersions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

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
    /// Checks `id` against [`HOSTNAME_ID_PATTERN`]; `system` is the system
    /// tenant.
    ///
    /// # Errors
    ///
    /// Returns an `invalid` [`OperationOutcomeError`] naming the rule `id`
    /// breaks.
    pub fn new(id: String) -> Result<Self, OperationOutcomeError> {
        if let Some(problem) = hostname_id_problem(&id) {
            return Err(invalid_hostname_id("tenant", &id, problem));
        }
        Ok(Self::unchecked(id))
    }

    /// A random id of `len` lowercase letters and digits (see
    /// [`random_hostname_id`]).
    #[must_use]
    pub fn random(len: usize) -> Self {
        Self::unchecked(random_hostname_id(len))
    }

    fn unchecked(id: String) -> Self {
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
        TenantId::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
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
    /// Checks `id` against [`HOSTNAME_ID_PATTERN`]; `system` is the system
    /// project.
    ///
    /// # Errors
    ///
    /// Returns an `invalid` [`OperationOutcomeError`] naming the rule `id`
    /// breaks.
    pub fn new(id: String) -> Result<Self, OperationOutcomeError> {
        if let Some(problem) = hostname_id_problem(&id) {
            return Err(invalid_hostname_id("project", &id, problem));
        }
        Ok(Self::unchecked(id))
    }

    /// A random id of `len` lowercase letters and digits (see
    /// [`random_hostname_id`]).
    #[must_use]
    pub fn random(len: usize) -> Self {
        Self::unchecked(random_hostname_id(len))
    }

    fn unchecked(id: String) -> Self {
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
        ProjectId::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
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
