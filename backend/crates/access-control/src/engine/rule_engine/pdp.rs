use haste_fhir_client::FHIRClient;
use haste_fhir_model::r4::generated::resources::AccessPolicyV2;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_pointer::Pointer;
use std::sync::Arc;

use crate::context::PolicyContext;

#[allow(unused)]
enum PermissionLevels {
    Deny,
    Undetermined,
    Allow,
}

#[allow(unused)]
fn resolve_variable<CTX, Client: FHIRClient<CTX, OperationOutcomeError>>(
    _context: Arc<PolicyContext<CTX, Client>>,
    _pointer: Pointer<AccessPolicyV2, AccessPolicyV2>,
) -> Result<(), OperationOutcomeError> {
    Ok(())
}

#[allow(unused)]
pub async fn evaluate<CTX, Client: FHIRClient<CTX, OperationOutcomeError>>(
    _policy_context: Arc<PolicyContext<CTX, Client>>,
    _policy: &AccessPolicyV2,
) -> Result<(), OperationOutcomeError> {
    Ok(())
}
