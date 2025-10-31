use oxidized_fhir_client::{FHIRClient, request::FHIRRequest};
use oxidized_fhir_model::r4::generated::{
    resources::AccessPolicyV2, terminology::AccessPolicyv2Engine,
};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_jwt::{ProjectId, TenantId};
use oxidized_reflect::{MetaValue, derive::Reflect};

mod engine;
mod utilities;

#[derive(Debug, Reflect)]
struct PolicyEnvironment {
    tenant: TenantId,
    project: ProjectId,
    request: FHIRRequest,
    user: Option<String>,
}

struct PolicyContext<CTX, Client: FHIRClient<CTX, OperationOutcomeError>> {
    client: Client,
    client_context: CTX,

    environment: PolicyEnvironment,
}

pub fn evaluate_policy<CTX, Client: FHIRClient<CTX, OperationOutcomeError>>(
    context: &PolicyContext<CTX, Client>,
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
