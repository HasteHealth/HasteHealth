use haste_fhir_model::r4::generated::{resources::ResourceType, terminology::IssueType};
use haste_fhir_operation_error::OperationOutcomeError;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum OIDCScope {
    OpenId,
    Profile,
    Email,
    OfflineAccess,
    OnlineAccess,
}

impl From<OIDCScope> for String {
    fn from(value: OIDCScope) -> Self {
        match value {
            OIDCScope::OpenId => "openid".to_string(),
            OIDCScope::Profile => "profile".to_string(),
            OIDCScope::Email => "email".to_string(),
            OIDCScope::OfflineAccess => "offline_access".to_string(),
            OIDCScope::OnlineAccess => "online_access".to_string(),
        }
    }
}

impl TryFrom<&str> for OIDCScope {
    type Error = OperationOutcomeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "openid" => Ok(Self::OpenId),
            "profile" => Ok(Self::Profile),
            "email" => Ok(Self::Email),
            "offline_access" => Ok(Self::OfflineAccess),
            "online_access" => Ok(Self::OnlineAccess),
            _ => Err(OperationOutcomeError::error(
                IssueType::not_supported(),
                format!("OIDC Scope '{value}' not supported."),
            )),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LaunchSystemScope;

impl From<LaunchSystemScope> for String {
    fn from(_: LaunchSystemScope) -> Self {
        "launch".to_string()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LaunchType {
    Encounter,
    Patient,
}

impl TryFrom<&str> for LaunchType {
    type Error = OperationOutcomeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "encounter" => Ok(LaunchType::Encounter),
            "patient" => Ok(LaunchType::Patient),
            _ => Err(OperationOutcomeError::error(
                IssueType::not_supported(),
                format!("Launch type '{value}' not supported."),
            )),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LaunchTypeScope {
    pub launch_type: LaunchType,
}

impl From<LaunchTypeScope> for String {
    fn from(value: LaunchTypeScope) -> Self {
        match value.launch_type {
            LaunchType::Encounter => "launch/encounter",
            LaunchType::Patient => "launch/patient",
        }
        .to_string()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SmartResourceScopeUser {
    User,
    System,
    Patient,
}

impl TryFrom<&str> for SmartResourceScopeUser {
    type Error = OperationOutcomeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "user" => Ok(SmartResourceScopeUser::User),
            "system" => Ok(SmartResourceScopeUser::System),
            "patient" => Ok(SmartResourceScopeUser::Patient),
            _ => Err(OperationOutcomeError::error(
                IssueType::not_supported(),
                format!("Smart resource scope level '{value}' not supported."),
            )),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SmartResourceScopeLevel {
    ResourceType(ResourceType),
    AllResources,
}

impl TryFrom<&str> for SmartResourceScopeLevel {
    type Error = OperationOutcomeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "*" => Ok(SmartResourceScopeLevel::AllResources),
            resource_type => {
                let resource_type = ResourceType::try_from(value).map_err(|_e| {
                    OperationOutcomeError::error(
                        IssueType::not_supported(),
                        format!(
                            "Smart resource scope resource type '{resource_type}' not supported.",
                        ),
                    )
                })?;
                Ok(SmartResourceScopeLevel::ResourceType(resource_type))
            }
        }
    }
}

impl SmartResourceScopeLevel {
    /// Whether this level reaches every resource type `other` names: `*`
    /// reaches them all, a resource type only itself.
    #[must_use]
    pub fn covers(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::AllResources, _) => true,
            (Self::ResourceType(_), Self::AllResources) => false,
            (Self::ResourceType(held), Self::ResourceType(wanted)) => held == wanted,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SmartResourceScopePermission {
    Create,
    Read,
    Update,
    Delete,
    Search,
}

/// How a scope's permissions are written. SMART App Launch v1 uses a word
/// (`read`, `write`, or `*` for both), v2 the letters `cruds`. A scope is
/// written back the way the client wrote it, so a v1 client is granted the
/// v1 strings it asked for.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum PermissionSyntax {
    V1,
    V2,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SmartResourceScopePermissions {
    permissions: Vec<SmartResourceScopePermission>,
    syntax: PermissionSyntax,
}

impl SmartResourceScopePermissions {
    /// Permissions written in v2 syntax.
    #[must_use]
    pub fn new(permissions: Vec<SmartResourceScopePermission>) -> Self {
        Self {
            permissions,
            syntax: PermissionSyntax::V2,
        }
    }

    /// Permissions written in v1 syntax (`read`, `write` or `*`).
    #[must_use]
    pub fn v1(permissions: Vec<SmartResourceScopePermission>) -> Self {
        Self {
            permissions,
            syntax: PermissionSyntax::V1,
        }
    }

    #[must_use]
    pub fn has_permission(&self, permission: &SmartResourceScopePermission) -> bool {
        self.permissions.contains(permission)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.permissions.is_empty()
    }

    pub fn add_permission(&mut self, permission: SmartResourceScopePermission) {
        if !self.has_permission(&permission) {
            self.permissions.push(permission);
        }
    }

    /// The permissions also in `kept`, written in the same syntax.
    #[must_use]
    pub fn restricted_to(&self, kept: &[SmartResourceScopePermission]) -> Self {
        Self {
            permissions: self
                .permissions
                .iter()
                .filter(|permission| kept.contains(permission))
                .cloned()
                .collect(),
            syntax: self.syntax,
        }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, SmartResourceScopePermission> {
        self.permissions.iter()
    }

    /// The v1 word for these permissions, if one names exactly them.
    fn v1_word(&self) -> Option<&'static str> {
        let has = |permission| self.has_permission(&permission);
        match (
            has(SmartResourceScopePermission::Create),
            has(SmartResourceScopePermission::Read),
            has(SmartResourceScopePermission::Update),
            has(SmartResourceScopePermission::Delete),
            has(SmartResourceScopePermission::Search),
        ) {
            (true, true, true, true, true) => Some("*"),
            (false, true, false, false, true) => Some("read"),
            (true, false, true, true, false) => Some("write"),
            _ => None,
        }
    }
}

impl<'a> IntoIterator for &'a SmartResourceScopePermissions {
    type Item = &'a SmartResourceScopePermission;
    type IntoIter = std::slice::Iter<'a, SmartResourceScopePermission>;

    fn into_iter(self) -> Self::IntoIter {
        self.permissions.iter()
    }
}

/// v1 permissions as their word, v2 as letters in `cruds` order. v1
/// permissions that no word names (a v1 scope narrowed that way) fall back to
/// letters.
impl From<&SmartResourceScopePermissions> for String {
    fn from(value: &SmartResourceScopePermissions) -> Self {
        if value.syntax == PermissionSyntax::V1
            && let Some(word) = value.v1_word()
        {
            return word.to_string();
        }
        [
            (SmartResourceScopePermission::Create, 'c'),
            (SmartResourceScopePermission::Read, 'r'),
            (SmartResourceScopePermission::Update, 'u'),
            (SmartResourceScopePermission::Delete, 'd'),
            (SmartResourceScopePermission::Search, 's'),
        ]
        .into_iter()
        .filter(|(permission, _)| value.has_permission(permission))
        .map(|(_, letter)| letter)
        .collect()
    }
}

static SMART_RESOURCE_SCOPE_PERMISSION_ORDER: &[char] = &['c', 'r', 'u', 'd', 's'];

impl TryFrom<&str> for SmartResourceScopePermissions {
    type Error = OperationOutcomeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "*" => Ok(SmartResourceScopePermissions::v1(vec![
                SmartResourceScopePermission::Create,
                SmartResourceScopePermission::Read,
                SmartResourceScopePermission::Update,
                SmartResourceScopePermission::Delete,
                SmartResourceScopePermission::Search,
            ])),
            "write" => Ok(SmartResourceScopePermissions::v1(vec![
                SmartResourceScopePermission::Create,
                SmartResourceScopePermission::Update,
                SmartResourceScopePermission::Delete,
            ])),
            "read" => Ok(SmartResourceScopePermissions::v1(vec![
                SmartResourceScopePermission::Read,
                SmartResourceScopePermission::Search,
            ])),
            methods => {
                let mut methods_obj = SmartResourceScopePermissions::new(vec![]);

                // Scope requests with undefined or out of order interactions MAY be ignored, replaced with server default scopes, or rejected.
                // per [https://build.fhir.org/ig/HL7/smart-app-launch/scopes-and-launch-context.html#scopes-for-requesting-fhir-resources].
                let mut current_index: i8 = -1;
                for method in methods.chars() {
                    let found_index = SMART_RESOURCE_SCOPE_PERMISSION_ORDER
                        .iter()
                        .position(|o| *o == method)
                        .map(i8::try_from)
                        .transpose()
                        .map_err(|e| {
                            OperationOutcomeError::error(
                                IssueType::not_supported(),
                                format!("Permission order index overflow {e}"),
                            )
                        })?;

                    if found_index <= Some(current_index) || found_index.is_none() {
                        return Err(OperationOutcomeError::error(
                            IssueType::not_supported(),
                            format!(
                                "Invalid scope access type methods: '{method}' not supported or in wrong place must be in 'cruds' order.",
                            ),
                        ));
                    }

                    current_index = found_index.unwrap_or(0);

                    match method {
                        /*
                         * Type level create
                         */
                        'c' => {
                            methods_obj.add_permission(SmartResourceScopePermission::Create);
                        }
                        /*
                         * Instance level read
                         * Instance level vread
                         * Instance level history
                         */
                        'r' => {
                            methods_obj.add_permission(SmartResourceScopePermission::Read);
                        }
                        /*
                         * Instance level update Note that some servers allow for an update operation to create a new instance,
                         * and this is allowed by the update scope
                         * Instance level patch
                         */
                        'u' => {
                            methods_obj.add_permission(SmartResourceScopePermission::Update);
                        }
                        /*
                         * Instance level delete
                         */
                        'd' => {
                            methods_obj.add_permission(SmartResourceScopePermission::Delete);
                        }
                        /*
                         * Type level search
                         * Type level history
                         * System level search
                         * System level history
                         */
                        's' => {
                            methods_obj.add_permission(SmartResourceScopePermission::Search);
                        }
                        _ => {}
                    }
                }

                Ok(methods_obj)
            }
        }
    }
}

/// One `name=value` of a granular scope's query, as written: the search
/// parameter and the value a resource must match on it.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ScopeConstraint {
    pub name: String,
    pub value: String,
}

impl ScopeConstraint {
    fn parse(pair: &str, scope: &str) -> Result<Self, OperationOutcomeError> {
        match pair.split_once('=') {
            Some((name, value)) if !name.is_empty() && !value.is_empty() => Ok(ScopeConstraint {
                name: name.to_string(),
                value: value.to_string(),
            }),
            _ => Err(OperationOutcomeError::error(
                IssueType::not_supported(),
                format!(
                    "Invalid smart resource scope: '{scope}'. Expected 'param=value' after '?'."
                ),
            )),
        }
    }
}

/// A SMART resource scope: who it is for, which type, which permissions and,
/// for a granular scope (`patient/Observation.rs?category=...`), the search
/// parameters a resource must match to be reached. A scope with no
/// constraints reaches every resource of its type.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SMARTResourceScope {
    pub user: SmartResourceScopeUser,
    pub level: SmartResourceScopeLevel,
    pub permissions: SmartResourceScopePermissions,
    /// The `?param=value&param=value` of a granular scope, in order; empty for
    /// a plain scope. A resource is reached only if it matches every one.
    pub constraints: Vec<ScopeConstraint>,
}

impl SMARTResourceScope {
    /// Whether this scope is granular: narrowed to the resources that match
    /// its search parameters.
    #[must_use]
    pub fn is_granular(&self) -> bool {
        !self.constraints.is_empty()
    }

    /// Whether this scope reaches every resource `wanted` reaches, whatever
    /// the permissions: the same context, a level that covers `wanted`'s,
    /// and no constraint `wanted` lacks. A plain scope reaches every
    /// resource of its type, so it covers each granular scope on it; a
    /// granular scope covers one with the same constraints or more.
    #[must_use]
    pub fn reaches(&self, wanted: &Self) -> bool {
        self.user == wanted.user
            && self.level.covers(&wanted.level)
            && self
                .constraints
                .iter()
                .all(|constraint| wanted.constraints.contains(constraint))
    }
}

impl From<SMARTResourceScope> for String {
    fn from(value: SMARTResourceScope) -> Self {
        let user_str = match value.user {
            SmartResourceScopeUser::User => "user",
            SmartResourceScopeUser::System => "system",
            SmartResourceScopeUser::Patient => "patient",
        };

        let level_str = match value.level {
            SmartResourceScopeLevel::AllResources => "*".to_string(),
            SmartResourceScopeLevel::ResourceType(resource_type) => {
                resource_type.as_ref().to_string()
            }
        };

        let permissions_str = String::from(&value.permissions);

        let mut text = format!("{user_str}/{level_str}.{permissions_str}");
        for (index, constraint) in value.constraints.iter().enumerate() {
            text.push(if index == 0 { '?' } else { '&' });
            text.push_str(&constraint.name);
            text.push('=');
            text.push_str(&constraint.value);
        }
        text
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SmartScope {
    LaunchSystem(LaunchSystemScope),
    LaunchType(LaunchTypeScope),
    Resource(SMARTResourceScope),
    FHIRUser,
}

impl From<SmartScope> for String {
    fn from(value: SmartScope) -> Self {
        match value {
            SmartScope::FHIRUser => "fhirUser".to_string(),
            SmartScope::LaunchSystem(launch_system) => String::from(launch_system),
            SmartScope::LaunchType(launch_type) => String::from(launch_type),
            SmartScope::Resource(resource) => String::from(resource),
        }
    }
}

impl TryFrom<&str> for SmartScope {
    type Error = OperationOutcomeError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "fhirUser" => Ok(SmartScope::FHIRUser),
            "launch" => Ok(SmartScope::LaunchSystem(LaunchSystemScope)),
            _ if value.starts_with("launch/") => {
                let chunks: Vec<&str> = value.split('/').collect();
                if chunks.len() != 2 {
                    return Err(OperationOutcomeError::error(
                        IssueType::not_supported(),
                        format!("Invalid launch scope: '{value}'."),
                    ));
                }

                let launch_type = LaunchType::try_from(chunks[1])?;

                Ok(SmartScope::LaunchType(LaunchTypeScope { launch_type }))
            }
            _ if value.starts_with("user/")
                || value.starts_with("system/")
                || value.starts_with("patient/") =>
            {
                // A granular scope's query comes after the first `?`. The
                // values can hold `/` and `.` (a code system URL), so the
                // scope proper is only what precedes it.
                let (scope, query) = match value.split_once('?') {
                    Some((scope, query)) => (scope, Some(query)),
                    None => (value, None),
                };
                let parts: Vec<&str> = scope.split('/').collect();
                if parts.len() != 2 {
                    return Err(OperationOutcomeError::error(
                        IssueType::not_supported(),
                        format!("Invalid smart resource scope: '{value}'."),
                    ));
                }

                let user = SmartResourceScopeUser::try_from(parts[0])?;
                let permissions_parts: Vec<&str> = parts[1].split('.').collect();
                if permissions_parts.len() != 2 {
                    return Err(OperationOutcomeError::error(
                        IssueType::not_supported(),
                        format!("Invalid smart resource scope: '{value}'."),
                    ));
                }

                let level = SmartResourceScopeLevel::try_from(permissions_parts[0])?;
                let permissions = SmartResourceScopePermissions::try_from(permissions_parts[1])?;
                let constraints = query
                    .map(|query| {
                        query
                            .split('&')
                            .map(|pair| ScopeConstraint::parse(pair, value))
                            .collect::<Result<Vec<_>, _>>()
                    })
                    .transpose()?
                    .unwrap_or_default();
                // A search parameter belongs to a resource type; there is
                // nothing for one to mean across all of them.
                if !constraints.is_empty() && level == SmartResourceScopeLevel::AllResources {
                    return Err(OperationOutcomeError::error(
                        IssueType::not_supported(),
                        format!(
                            "Invalid smart resource scope: '{value}'. Search parameters need a resource type, not '*'."
                        ),
                    ));
                }

                Ok(SmartScope::Resource(SMARTResourceScope {
                    user,
                    level,
                    permissions,
                    constraints,
                }))
            }
            _ => Err(OperationOutcomeError::error(
                IssueType::not_supported(),
                format!("Smart Scope '{value}' not supported."),
            )),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Scope {
    OIDC(OIDCScope),
    SMART(SmartScope),
}

impl TryFrom<&str> for Scope {
    type Error = OperationOutcomeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let Ok(oidc_scope) = OIDCScope::try_from(value) {
            Ok(Self::OIDC(oidc_scope))
        } else {
            Ok(Self::SMART(SmartScope::try_from(value)?))
        }
    }
}

impl From<Scope> for String {
    fn from(value: Scope) -> Self {
        match value {
            Scope::OIDC(oidc_scope) => String::from(oidc_scope),
            Scope::SMART(smart_scope) => String::from(smart_scope),
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Scopes(pub Vec<Scope>);

impl Scopes {
    #[must_use]
    pub fn contains_scope(&self, scope: &Scope) -> bool {
        self.0.contains(scope)
    }

    /// Whether these scopes grant everything `scope` grants.
    ///
    /// A SMART resource scope is covered when the scopes of the same context
    /// (`patient/`, `user/` or `system/`) that reach its resource type (`*`
    /// reaches every type) together carry each of its permissions. So
    /// `patient/*.rs` covers `patient/Medication.rs`, and `patient/Medication.rs`
    /// covers `patient/Medication.s`: per SMART App Launch, "Observation.rs"
    /// is "Observation.r Observation.s", and scopes combine into a union of
    /// access. A permission never implies another (write does not imply
    /// read), so `patient/Medication.cud` does not cover `patient/Medication.r`.
    /// A granular scope is covered by the plain scope on its type and by a
    /// granular one with the same constraints (or fewer): see
    /// [`SMARTResourceScope::reaches`]. Any other scope is covered only by
    /// itself.
    #[must_use]
    pub fn covers_scope(&self, scope: &Scope) -> bool {
        let Scope::SMART(SmartScope::Resource(wanted)) = scope else {
            return self.contains_scope(scope);
        };
        let reaching = || {
            self.0.iter().filter_map(|held| match held {
                Scope::SMART(SmartScope::Resource(held)) if held.reaches(wanted) => {
                    Some(&held.permissions)
                }
                _ => None,
            })
        };
        wanted
            .permissions
            .iter()
            .all(|permission| reaching().any(|held| held.has_permission(permission)))
    }
}

impl TryFrom<&str> for Scopes {
    type Error = OperationOutcomeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let scopes: Result<Vec<Scope>, OperationOutcomeError> =
            value.split_whitespace().map(Scope::try_from).collect();

        Ok(Scopes(scopes?))
    }
}

// Used by sqlx binding this is not safe.
impl From<String> for Scopes {
    fn from(value: String) -> Self {
        Self::try_from(value.as_str()).expect("Invalid scopes string")
    }
}

impl<'de> Deserialize<'de> for Scopes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Scopes::try_from(s.as_str()).map_err(serde::de::Error::custom)
    }
}

impl From<Scopes> for String {
    fn from(value: Scopes) -> Self {
        value
            .0
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl Serialize for Scopes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&String::from(self.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use haste_fhir_model::r4::generated::resources::ResourceType;

    #[test]
    fn test_multiple_correct() {
        assert_eq!(
            Scopes::try_from("openid profile email offline_access launch/patient user/*.*")
                .unwrap(),
            Scopes(vec![
                Scope::OIDC(OIDCScope::OpenId),
                Scope::OIDC(OIDCScope::Profile),
                Scope::OIDC(OIDCScope::Email),
                Scope::OIDC(OIDCScope::OfflineAccess),
                Scope::SMART(SmartScope::LaunchType(LaunchTypeScope {
                    launch_type: LaunchType::Patient,
                })),
                Scope::SMART(SmartScope::Resource(SMARTResourceScope {
                    user: SmartResourceScopeUser::User,
                    level: SmartResourceScopeLevel::AllResources,
                    permissions: SmartResourceScopePermissions::v1(vec![
                        SmartResourceScopePermission::Create,
                        SmartResourceScopePermission::Read,
                        SmartResourceScopePermission::Update,
                        SmartResourceScopePermission::Delete,
                        SmartResourceScopePermission::Search,
                    ]),
                    constraints: vec![],
                })),
            ]),
        );

        assert_eq!(
            Scopes::try_from("launch/encounter   system/Patient.cud").unwrap(),
            Scopes(vec![
                Scope::SMART(SmartScope::LaunchType(LaunchTypeScope {
                    launch_type: LaunchType::Encounter,
                })),
                Scope::SMART(SmartScope::Resource(SMARTResourceScope {
                    user: SmartResourceScopeUser::System,
                    level: SmartResourceScopeLevel::ResourceType(ResourceType::Patient),
                    permissions: SmartResourceScopePermissions::new(vec![
                        SmartResourceScopePermission::Create,
                        SmartResourceScopePermission::Update,
                        SmartResourceScopePermission::Delete,
                    ]),
                    constraints: vec![],
                })),
            ]),
        );
    }

    const LABORATORY: &str = "patient/Observation.rs?category=http://terminology.hl7.org/CodeSystem/observation-category|laboratory";
    const VITAL_SIGNS: &str = "patient/Observation.rs?category=http://terminology.hl7.org/CodeSystem/observation-category|vital-signs";

    #[test]
    fn granular_scope_parses_and_formats() {
        let parsed = Scopes::try_from(LABORATORY).unwrap();
        assert_eq!(
            parsed,
            Scopes(vec![Scope::SMART(SmartScope::Resource(
                SMARTResourceScope {
                    user: SmartResourceScopeUser::Patient,
                    level: SmartResourceScopeLevel::ResourceType(ResourceType::Observation),
                    permissions: SmartResourceScopePermissions::new(vec![
                        SmartResourceScopePermission::Read,
                        SmartResourceScopePermission::Search,
                    ]),
                    constraints: vec![ScopeConstraint {
                        name: "category".to_string(),
                        value:
                            "http://terminology.hl7.org/CodeSystem/observation-category|laboratory"
                                .to_string(),
                    }],
                }
            ))])
        );
        assert_eq!(String::from(parsed), LABORATORY);

        let two = "user/Observation.rs?category=laboratory&date=gt2020";
        assert_eq!(String::from(Scopes::try_from(two).unwrap()), two);
    }

    #[test]
    fn granular_scope_needs_param_value_pairs_and_a_type() {
        assert!(Scopes::try_from("patient/Observation.rs?category").is_err());
        assert!(Scopes::try_from("patient/Observation.rs?=laboratory").is_err());
        assert!(Scopes::try_from("patient/Observation.rs?category=").is_err());
        assert!(Scopes::try_from("patient/Observation.rs?").is_err());
        assert!(Scopes::try_from("patient/*.rs?category=laboratory").is_err());
    }

    #[test]
    fn plain_scope_covers_granular_not_the_reverse() {
        let plain = Scopes::try_from("patient/Observation.rs").unwrap();
        let laboratory = Scopes::try_from(LABORATORY).unwrap();
        let wanted = &laboratory.0[0];
        assert!(plain.covers_scope(wanted));
        assert!(laboratory.covers_scope(wanted));
        assert!(!laboratory.covers_scope(&plain.0[0]));
        assert!(!Scopes::try_from(VITAL_SIGNS).unwrap().covers_scope(wanted));
        // More constraints is narrower, so the one-constraint scope covers it.
        let narrower = Scopes::try_from(format!("{LABORATORY}&date=gt2020").as_str()).unwrap();
        assert!(laboratory.covers_scope(&narrower.0[0]));
        assert!(!narrower.covers_scope(wanted));
        // Permissions still have to be carried.
        assert!(
            !Scopes::try_from("patient/Observation.r")
                .unwrap()
                .covers_scope(wanted)
        );
    }

    #[test]
    fn invalid_order() {
        assert!(Scopes::try_from("launch/encounter   system/Patient.duc").is_err());
    }

    #[test]
    fn invalid_system() {
        assert!(Scopes::try_from("launch/encounter   sytem/Patient.cud").is_err());
    }

    #[test]
    fn unknown_scope() {
        assert!(Scopes::try_from("badscope  sytem/Patient.cud").is_err());
    }

    #[test]
    fn test_roundtrip() {
        assert_eq!(
            String::from(
                Scopes::try_from(
                    "openid profile email offline_access launch/patient user/*.* patient/Patient.read user/Observation.write"
                )
                .unwrap()
            ),
            "openid profile email offline_access launch/patient user/*.* patient/Patient.read user/Observation.write".to_string(),
        );

        assert_eq!(
            String::from(Scopes::try_from("launch/encounter system/Patient.cud").unwrap()),
            "launch/encounter system/Patient.cud".to_string()
        );
    }
}
