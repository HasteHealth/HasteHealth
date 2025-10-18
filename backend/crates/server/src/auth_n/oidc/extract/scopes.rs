use crate::auth_n::oidc::middleware::OIDCParameters;
use axum::{
    Extension, RequestPartsExt,
    extract::FromRequestParts,
    http::request::Parts,
    response::{IntoResponse, Response},
};
use oxidized_fhir_model::r4::generated::{resources::ResourceType, terminology::IssueType};
use oxidized_fhir_operation_error::OperationOutcomeError;
use std::borrow::Cow;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum OIDCScope {
    OpenId,
    Profile,
    Email,
    OfflineAccess,
    OnlineAccess,
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
                IssueType::NotSupported(None),
                format!("OIDC Scope '{}' not supported.", value),
            )),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LaunchSystemScope;

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
                IssueType::NotSupported(None),
                format!("Launch type '{}' not supported.", value),
            )),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LaunchTypeScope {
    pub launch_type: LaunchType,
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
                IssueType::NotSupported(None),
                format!("Smart resource scope level '{}' not supported.", value),
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
                        IssueType::NotSupported(None),
                        format!(
                            "Smart resource scope resource type '{}' not supported.",
                            resource_type,
                        ),
                    )
                })?;
                Ok(SmartResourceScopeLevel::ResourceType(resource_type))
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SmartResourceScopePermissions {
    pub create: bool,
    pub read: bool,
    pub update: bool,
    pub delete: bool,
    pub search: bool,
}

static SMART_RESOURCE_SCOPE_PERMISSION_ORDER: &[char] = &['c', 'r', 'u', 'd', 's'];

impl TryFrom<&str> for SmartResourceScopePermissions {
    type Error = OperationOutcomeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "*" => Ok(SmartResourceScopePermissions {
                create: true,
                read: true,
                update: true,
                delete: true,
                search: true,
            }),
            "write" => Ok(SmartResourceScopePermissions {
                create: true,
                update: true,
                delete: true,
                read: false,
                search: false,
            }),
            "read" => Ok(SmartResourceScopePermissions {
                read: true,
                search: true,
                create: false,
                update: false,
                delete: false,
            }),
            methods => {
                let mut methods_obj = SmartResourceScopePermissions {
                    create: false,
                    read: false,
                    update: false,
                    delete: false,
                    search: false,
                };

                // Scope requests with undefined or out of order interactions MAY be ignored, replaced with server default scopes, or rejected.
                // per [https://build.fhir.org/ig/HL7/smart-app-launch/scopes-and-launch-context.html#scopes-for-requesting-fhir-resources].
                let mut current_index: i8 = -1;
                for method in methods.chars() {
                    let found_index = SMART_RESOURCE_SCOPE_PERMISSION_ORDER
                        .iter()
                        .position(|o| *o == method)
                        .map(|p| p as i8);

                    if found_index <= Some(current_index) || found_index.is_none() {
                        return Err(OperationOutcomeError::error(
                            IssueType::NotSupported(None),
                            format!(
                                "Invalid scope access type methods: '{}' not supported or in wrong place must be in 'cruds' order.",
                                method
                            ),
                        ));
                    }

                    current_index = found_index.unwrap_or(0);

                    match method {
                        /*
                         * Type level create
                         */
                        'c' => {
                            methods_obj.create = true;
                        }
                        /*
                         * Instance level read
                         * Instance level vread
                         * Instance level history
                         */
                        'r' => {
                            methods_obj.read = true;
                        }
                        /*
                         * Instance level update Note that some servers allow for an update operation to create a new instance,
                         * and this is allowed by the update scope
                         * Instance level patch
                         */
                        'u' => {
                            methods_obj.update = true;
                        }
                        /*
                         * Instance level delete
                         */
                        'd' => {
                            methods_obj.delete = true;
                        }
                        /*
                         * Type level search
                         * Type level history
                         * System level search
                         * System level history
                         */
                        's' => {
                            methods_obj.search = true;
                        }
                        _ => {}
                    }
                }

                Ok(methods_obj)
            }
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SMARTResourceScope {
    pub user: SmartResourceScopeUser,
    pub level: SmartResourceScopeLevel,
    pub permissions: SmartResourceScopePermissions,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SmartScope {
    LaunchSystem(LaunchSystemScope),
    LaunchType(LaunchTypeScope),
    Resource(SMARTResourceScope),
    FHIRUser,
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
                        IssueType::NotSupported(None),
                        format!("Invalid launch scope: '{}'.", value),
                    ));
                }

                let launch_type = LaunchType::try_from(chunks[1])?;

                Ok(SmartScope::LaunchType(LaunchTypeScope { launch_type }))
            }
            _ if value.starts_with("user/")
                || value.starts_with("system/")
                || value.starts_with("patient/") =>
            {
                let parts: Vec<&str> = value.split('/').collect();
                if parts.len() != 2 {
                    return Err(OperationOutcomeError::error(
                        IssueType::NotSupported(None),
                        format!("Invalid smart resource scope: '{}'.", value),
                    ));
                }

                let user = SmartResourceScopeUser::try_from(parts[0])?;
                let permissions_parts: Vec<&str> = parts[1].split('.').collect();
                if permissions_parts.len() != 2 {
                    return Err(OperationOutcomeError::error(
                        IssueType::NotSupported(None),
                        format!("Invalid smart resource scope: '{}'.", value),
                    ));
                }

                let level = SmartResourceScopeLevel::try_from(permissions_parts[0])?;
                let permissions = SmartResourceScopePermissions::try_from(permissions_parts[1])?;

                Ok(SmartScope::Resource(SMARTResourceScope {
                    user,
                    level,
                    permissions,
                }))
            }
            _ => Err(OperationOutcomeError::error(
                IssueType::NotSupported(None),
                format!("Smart Scope '{}' not supported.", value),
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Scopes(pub Vec<Scope>);

impl TryFrom<&str> for Scopes {
    type Error = OperationOutcomeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let scopes: Result<Vec<Scope>, OperationOutcomeError> = value
            .split_whitespace()
            .map(|s| Scope::try_from(s))
            .collect();

        Ok(Scopes(scopes?))
    }
}

impl<S: Send + Sync> FromRequestParts<S> for Scopes {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let Extension(oidc_params) = parts
            .extract::<Extension<OIDCParameters>>()
            .await
            .map_err(|err| err.into_response())?;

        let scope = oidc_params
            .parameters
            .get("scope")
            .map(|s| Cow::Borrowed(s))
            .unwrap_or_else(|| Cow::Owned("".to_string()));

        let scopes = Scopes::try_from(scope.as_str()).map_err(|err| err.into_response())?;

        Ok(scopes)
    }
}
