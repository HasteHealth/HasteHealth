use haste_fhir_client::FHIRClient;
use haste_fhir_model::r4::generated::resources::{AccessPolicyV2, AccessPolicyV2Rule};
use haste_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use haste_pointer::Pointer;
use std::sync::Arc;

use crate::context::{PermissionLevel, PermissionLevelError, PolicyContext};

#[derive(Debug, OperationOutcomeError)]
pub enum PDPError {
    #[error(code = "exception", diagnostic = "Pointer at '{arg0}' failed.")]
    PointerError(String),
    #[error(code = "invalid", diagnostic = "{arg0:?}")]
    InvalidPermissionLevel(PermissionLevelError),
}

fn get_max(p1: &PermissionLevel, p2: &PermissionLevel) -> Result<PermissionLevel, PDPError> {
    let max = std::cmp::max(i8::from(p1), i8::from(p2));

    PermissionLevel::try_from(max).map_err(PDPError::InvalidPermissionLevel)
}

#[allow(unused)]
fn resolve_variable<CTX, Client: FHIRClient<CTX, OperationOutcomeError>>(
    _context: Arc<PolicyContext<CTX, Client>>,
    _pointer: Pointer<AccessPolicyV2, AccessPolicyV2>,
) -> Result<(), OperationOutcomeError> {
    Ok(())
}

async fn evaluate_access_policy_rule<'a, CTX, Client: FHIRClient<CTX, OperationOutcomeError>>(
    policy_context: Arc<PolicyContext<CTX, Client>>,
    _rule_pointer: Pointer<'a, AccessPolicyV2, AccessPolicyV2Rule>,
) -> Result<(Arc<PolicyContext<CTX, Client>>, PermissionLevel), PDPError> {
    Ok((policy_context, PermissionLevel::Deny))
}

#[allow(unused)]
pub async fn evaluate<CTX, Client: FHIRClient<CTX, OperationOutcomeError>>(
    mut policy_context: Arc<PolicyContext<CTX, Client>>,
    policy: &AccessPolicyV2,
) -> Result<PermissionLevel, PDPError> {
    let pointer = Pointer::<AccessPolicyV2, AccessPolicyV2>::new(policy);
    let rules_pointer = pointer
        .descend::<Option<Vec<AccessPolicyV2Rule>>>(&haste_pointer::Key::Field("rule".to_string()))
        .ok_or_else(|| PDPError::PointerError("rule".to_string()))?;

    let mut result = PermissionLevel::Deny;

    for (index, _) in policy.rule.as_ref().unwrap_or(&vec![]).iter().enumerate() {
        let rule_pointer = rules_pointer
            .descend::<AccessPolicyV2Rule>(&haste_pointer::Key::Index(index))
            .ok_or_else(|| PDPError::PointerError(format!("{}/{}", rules_pointer.path(), index)))?;

        match evaluate_access_policy_rule(policy_context.clone(), rule_pointer).await? {
            (_, PermissionLevel::Deny) => return Ok(PermissionLevel::Deny),
            (context, permission_level) => {
                // Continue evaluating other rules
                policy_context = context;

                result = get_max(&result, &permission_level)?;
            }
        }
    }

    Ok(result)
}
