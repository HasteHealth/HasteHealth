use crate::request_reflection::RequestReflection;
use haste_fhir_client::{
    FHIRClient,
    request::{FHIRRequest, FHIRResponse},
};
use haste_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use haste_fhirpath::FPEngine;
use haste_jwt::{ProjectId, TenantId};
use haste_reflect::{MetaValue, derive::Reflect};
use std::{collections::HashMap, sync::Arc};

#[derive(PartialEq, Eq, Debug)]
pub enum PermissionLevel {
    Deny,
    Undetermined,
    Allow,
}

impl From<&PermissionLevel> for i8 {
    fn from(level: &PermissionLevel) -> Self {
        match level {
            PermissionLevel::Deny => -1,
            PermissionLevel::Undetermined => 0,
            PermissionLevel::Allow => 1,
        }
    }
}

#[derive(Debug, OperationOutcomeError)]
pub enum PermissionLevelError {
    #[error(
        code = "invalid",
        diagnostic = "Invalid permission level value: '{arg0}'."
    )]
    InvalidPermissionLevel(i8),
}

impl TryFrom<i8> for PermissionLevel {
    type Error = PermissionLevelError;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        match value {
            -1 => Ok(PermissionLevel::Deny),
            0 => Ok(PermissionLevel::Undetermined),
            1 => Ok(PermissionLevel::Allow),
            _ => Err(PermissionLevelError::InvalidPermissionLevel(value)),
        }
    }
}

#[derive(Debug, Reflect)]
pub struct UserInfo {
    pub id: String,
}

#[derive(Debug)]
pub struct PolicyEnvironment {
    pub tenant: TenantId,
    pub project: ProjectId,
    pub request: Arc<RequestReflection>,
    pub user: Arc<UserInfo>,
}

impl PolicyEnvironment {
    pub fn new(
        tenant: TenantId,
        project: ProjectId,
        request: FHIRRequest,
        user: Arc<UserInfo>,
    ) -> Self {
        Self {
            tenant,
            project,
            request: Arc::new(RequestReflection::from(request)),
            user,
        }
    }
}

pub struct PolicyContext<CTX, Client: FHIRClient<CTX, OperationOutcomeError>> {
    pub fp_engine: haste_fhirpath::FPEngine,
    pub client: Arc<Client>,
    pub client_context: CTX,

    #[allow(dead_code)]
    attributes_cache: HashMap<String, FHIRResponse>,
    pub environment: PolicyEnvironment,
}

impl<CTX, Client: FHIRClient<CTX, OperationOutcomeError>> PolicyContext<CTX, Client> {
    pub fn new(client: Arc<Client>, client_context: CTX, environment: PolicyEnvironment) -> Self {
        Self {
            fp_engine: FPEngine::new(),
            client,
            client_context,
            attributes_cache: HashMap::new(),
            environment,
        }
    }
}
