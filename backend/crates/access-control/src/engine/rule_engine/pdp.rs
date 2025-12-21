use haste_fhir_client::FHIRClient;
use haste_fhir_model::r4::generated::resources::AccessPolicyV2;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_pointer::Pointer;
use std::sync::Arc;

use crate::context::PolicyContext;

enum PermissionLevels {
    Deny,
    Undetermined,
    Allow,
}

fn resolve_variable<'a, CTX, Client: FHIRClient<CTX, OperationOutcomeError>>(
    context: Arc<PolicyContext<CTX, Client>>,
    pointer: Pointer<'a>,
) -> Result<(), OperationOutcomeError> {
}

pub async fn evaluate<CTX, Client: FHIRClient<CTX, OperationOutcomeError>>(
    _policy_context: Arc<PolicyContext<CTX, Client>>,
    _policy: &AccessPolicyV2,
) -> Result<(), OperationOutcomeError> {
    Ok(())
}
