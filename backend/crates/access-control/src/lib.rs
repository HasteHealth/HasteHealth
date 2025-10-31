use oxidized_fhir_model::r4::generated::{
    resources::AccessPolicyV2, terminology::AccessPolicyv2Engine,
};
use oxidized_fhir_operation_error::OperationOutcomeError;

mod engine;
mod utilities;

pub fn evaluate_policy(policy: &AccessPolicyV2) -> Result<(), OperationOutcomeError> {
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
