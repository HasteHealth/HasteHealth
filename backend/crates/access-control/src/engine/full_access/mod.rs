use haste_fhir_model::r4::generated::{
    resources::AccessPolicyV2, terminology::AccessPolicyv2Engine,
};
use haste_fhir_operation_error::OperationOutcomeError;

pub fn evaluate(policy: &AccessPolicyV2) -> Result<(), OperationOutcomeError> {
    // Sanity check to ensure we are only evaluating FullAccess policies here.
    // Note this is done on root lib.rs evaluation of policy.
    if let AccessPolicyv2Engine::FullAccess(_) = *policy.engine {
        Ok(())
    } else {
        Err(OperationOutcomeError::fatal(
            haste_fhir_model::r4::generated::terminology::IssueType::Forbidden(None),
            "Access policy denies access.".to_string(),
        ))
    }
}
