use oxidized_fhir_client::{FHIRClient, request::FHIRRequest};
use oxidized_fhir_model::r4::generated::{
    resources::AccessPolicyV2, terminology::AccessPolicyv2Engine,
};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_jwt::{ProjectId, TenantId, claims::UserTokenClaims};

mod engine;
mod utilities;

#[derive(Debug)]
struct PolicyEnvironment<'a> {
    tenant: &'a TenantId,
    project: &'a ProjectId,
    request: &'a FHIRRequest,
    user: &'a UserTokenClaims,
}

struct PolicyContext<'a, CTX, Client: FHIRClient<CTX, OperationOutcomeError>> {
    client: Client,
    client_context: CTX,

    environment: PolicyEnvironment<'a>,
}

pub fn evaluate_policy<'a, CTX, Client: FHIRClient<CTX, OperationOutcomeError>>(
    context: &PolicyContext<'a, CTX, Client>,
    policy: &AccessPolicyV2,
) -> Result<(), OperationOutcomeError> {
    match &*policy.engine {
        AccessPolicyv2Engine::FullAccess(_) => engine::full_access::evaluate(policy),
        AccessPolicyv2Engine::RuleEngine(_) => Err(OperationOutcomeError::fatal(
            oxidized_fhir_model::r4::generated::terminology::IssueType::Forbidden(None),
            "Access policy denies access.".to_string(),
        )),
        AccessPolicyv2Engine::Null(_) => Err(OperationOutcomeError::fatal(
            oxidized_fhir_model::r4::generated::terminology::IssueType::Forbidden(None),
            "Access policy denies access.".to_string(),
        )),
    }
}
