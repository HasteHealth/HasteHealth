use haste_fhir_client::{
    FHIRClient,
    request::{FHIRRequest, FHIRResponse},
};
use haste_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use haste_jwt::{ProjectId, TenantId, claims::UserTokenClaims};
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

#[derive(Debug)]
pub struct PolicyEnvironment {
    pub tenant: TenantId,
    pub project: ProjectId,
    pub request: FHIRRequest,
    pub user: Arc<UserTokenClaims>,
}

pub struct PolicyContext<CTX, Client: FHIRClient<CTX, OperationOutcomeError>> {
    pub fp_engine: haste_fhirpath::FPEngine,
    pub client: Arc<Client>,
    pub client_context: CTX,

    pub attributes: HashMap<String, FHIRResponse>,
    pub environment: PolicyEnvironment,
}
