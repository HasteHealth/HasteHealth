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

pub async fn evaluate_policy<
    'a,
    CTX: Send + Sync + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + Send + Sync + 'static,
>(
    context: Arc<context::PolicyContext<CTX, Client>>,
    policy: Arc<AccessPolicyV2>,
) -> Result<PermissionLevel, OperationOutcomeError> {
    match &*policy.engine {
        AccessPolicyv2Engine::FullAccess(_) => engine::full_access::evaluate(policy.as_ref()).await,
        AccessPolicyv2Engine::RuleEngine(_) => {
            Ok(engine::rule_engine::pdp::evaluate(context, policy).await?)
        }
        AccessPolicyv2Engine::Null(_) => Err(OperationOutcomeError::fatal(
            haste_fhir_model::r4::generated::terminology::IssueType::Forbidden(None),
            "Access policy denies access.".to_string(),
        )),
    }
}

pub fn evaluate_policies<
    CTX: Send + Sync + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + Send + Sync + 'static,
>(
    context: context::PolicyContext<CTX, Client>,
    policies: &Vec<Arc<AccessPolicyV2>>,
) -> impl Future<Output = Result<context::PolicyContext<CTX, Client>, OperationOutcomeError>> {
    async move {
        let mut outcomes = vec![];
        let context = Arc::new(context);

        for policy in policies {
            let result = evaluate_policy(context.clone(), policy.clone()).await;
            if let Ok(permission) = result {
                match permission {
                    PermissionLevel::Allow => {
                        return Arc::into_inner(context).ok_or_else(|| {
                            OperationOutcomeError::error(
                                IssueType::Forbidden(None),
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
            IssueType::Forbidden(None),
            format!("No policy has granted access to your request."),
        ))
    }
}
