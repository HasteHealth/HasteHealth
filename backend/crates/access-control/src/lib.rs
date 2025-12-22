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
mod utilities;

pub async fn evaluate_policy<'a, CTX, Client: FHIRClient<CTX, OperationOutcomeError>>(
    context: Arc<context::PolicyContext<CTX, Client>>,
    policy: &AccessPolicyV2,
) -> Result<PermissionLevel, OperationOutcomeError> {
    match &*policy.engine {
        AccessPolicyv2Engine::FullAccess(_) => engine::full_access::evaluate(policy).await,
        AccessPolicyv2Engine::RuleEngine(_) => {
            Ok(engine::rule_engine::pdp::evaluate(context, policy).await?)
        }
        AccessPolicyv2Engine::Null(_) => Err(OperationOutcomeError::fatal(
            haste_fhir_model::r4::generated::terminology::IssueType::Forbidden(None),
            "Access policy denies access.".to_string(),
        )),
    }
}

pub async fn evaluate_policies<CTX, Client: FHIRClient<CTX, OperationOutcomeError>>(
    context: context::PolicyContext<CTX, Client>,
    policies: &Vec<AccessPolicyV2>,
) -> Result<context::PolicyContext<CTX, Client>, OperationOutcomeError> {
    let mut outcomes = vec![];
    let context = Arc::new(context);
    for policy in policies {
        if let Err(e) = evaluate_policy(context.clone(), policy).await {
            outcomes.push(e);
        } else {
            return Arc::into_inner(context).ok_or_else(|| {
                OperationOutcomeError::error(
                    IssueType::Forbidden(None),
                    "Failed to retrieve policy context.".to_string(),
                )
            });
        }
    }

    Err(OperationOutcomeError::error(
        IssueType::Forbidden(None),
        format!("No policy has granted access to your request."),
    ))
}
