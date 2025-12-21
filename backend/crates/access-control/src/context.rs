use haste_fhir_client::{
    FHIRClient,
    request::{FHIRRequest, FHIRResponse},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId, claims::UserTokenClaims};
use std::{collections::HashMap, sync::Arc};

#[allow(unused)]
#[derive(Debug)]
pub struct PolicyEnvironment {
    pub tenant: TenantId,
    pub project: ProjectId,
    pub request: FHIRRequest,
    pub user: Arc<UserTokenClaims>,
}

#[derive(Debug)]
pub struct PolicyContext<CTX, Client: FHIRClient<CTX, OperationOutcomeError>> {
    pub client: Arc<Client>,
    pub client_context: CTX,

    pub attributes: HashMap<String, FHIRResponse>,
    pub environment: PolicyEnvironment,
}
