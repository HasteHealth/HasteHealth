use crate::context::PermissionLevel;
use haste_fhir_client::FHIRClient;
use haste_fhir_model::r4::generated::{
    resources::AccessPolicyV2,
    terminology::{AccessPolicyv2Engine, IssueType},
};
use haste_fhir_operation_error::OperationOutcomeError;
use std::sync::Arc;

pub mod context;
mod engine;
mod request_reflection;
mod utilities;

/// Evaluates an access policy using the configured policy engine.
///
/// # Errors
///
/// Returns an [`OperationOutcomeError`] when:
/// - the policy engine denies access,
/// - the rule engine fails while evaluating rules,
/// - the policy contains an invalid or unsupported configuration.
pub async fn evaluate_policy<
    CTX: Send + Sync + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + Send + Sync + 'static,
>(
    context: Arc<context::PolicyContext<CTX, Client>>,
    policy: Arc<AccessPolicyV2>,
) -> Result<PermissionLevel, OperationOutcomeError> {
    match &policy.engine {
        policy_engine if policy_engine == &AccessPolicyv2Engine::FULL_ACCESS => {
            Ok(engine::full_access::evaluate(policy.as_ref()))
        }
        policy_engine if policy_engine == &AccessPolicyv2Engine::RULE_ENGINE => {
            Ok(engine::rule_engine::pdp::evaluate(context, policy).await?)
        }
        policy_engine if policy_engine == &AccessPolicyv2Engine::NULL => {
            Err(OperationOutcomeError::fatal(
                haste_fhir_model::r4::generated::terminology::IssueType::FORBIDDEN,
                "Access policy denies access.".to_string(),
            ))
        }
        _ => Err(OperationOutcomeError::fatal(
            haste_fhir_model::r4::generated::terminology::IssueType::INVALID,
            "Unsupported policy engine.".to_string(),
        )),
    }
}

/// Evaluates a list of access policies and returns the updated policy context
/// when access is granted.
///
/// Policies are evaluated in order. The first policy returning
/// [`PermissionLevel::Allow`] grants access.
///
/// # Errors
///
/// Returns [`OperationOutcomeError`] when:
/// - no policy grants access,
/// - the evaluated policy returns an evaluation error,
/// - the policy context cannot be recovered after granting access.
pub async fn evaluate_policies<
    CTX: Send + Sync + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + Send + Sync + 'static,
>(
    context: context::PolicyContext<CTX, Client>,
    policies: &Vec<Arc<AccessPolicyV2>>,
) -> Result<context::PolicyContext<CTX, Client>, OperationOutcomeError> {
    let mut outcomes = vec![];
    let context = Arc::new(context);

    for policy in policies {
        let result = evaluate_policy(context.clone(), policy.clone()).await;
        if let Ok(permission) = result {
            match permission {
                PermissionLevel::Allow => {
                    return Arc::into_inner(context).ok_or_else(|| {
                        OperationOutcomeError::error(
                            IssueType::FORBIDDEN,
                            "Failed to retrieve policy context.".to_string(),
                        )
                    });
                }
                _ => {}
            }
        } else if let Err(e) = result {
            outcomes.push(e);
        }
    }

    Err(OperationOutcomeError::error(
        IssueType::FORBIDDEN,
        format!("No policy has granted access to your request."),
    ))
}
