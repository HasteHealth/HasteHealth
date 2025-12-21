//! Policy Information Point (PIP) module for the access control engine.
//! This module is responsible for retrieving contextual information that can be used during policy evaluation.
use haste_fhir_client::FHIRClient;
use haste_fhir_model::r4::generated::resources::AccessPolicyV2;
use haste_fhir_operation_error::OperationOutcomeError;
use std::sync::Arc;

use crate::context::PolicyContext;

pub struct PIPResult {}

#[allow(unused)]
pub async fn pip<'a, CTX, Client: FHIRClient<CTX, OperationOutcomeError>>(
    _policy_context: Arc<PolicyContext<CTX, Client>>,
    _policy: &AccessPolicyV2,
    variable_id: &str,
) -> Result<PIPResult, OperationOutcomeError> {
    match variable_id {
        "user" => None,
        _ => Some("test".to_string()),
    };

    Ok(PIPResult {})
}
