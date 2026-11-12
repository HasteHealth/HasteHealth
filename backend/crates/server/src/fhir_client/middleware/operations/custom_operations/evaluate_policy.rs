use crate::fhir_client::{
    ServerCTX,
    batch_transaction_processing::bundle_entry_to_fhir_request,
    middleware::{Backend, operations::ServerOperationContext},
};
use haste_access_control::{
    context::{PermissionLevel, PolicyContext, PolicyEnvironment, UserInfo},
    request_reflection::{Channel, RequestReflection},
    v3,
};
use haste_fhir_client::request::{FHIRRequest, InvocationRequest};
use haste_fhir_generated_ops::generated::HasteHealthEvaluatePolicy;
use haste_repository::fhir::FHIRRepository;
use std::sync::Arc;

use haste_fhir_model::r4::generated::{
    resources::{
        AccessPolicyV2, AccessPolicyV3, OperationOutcome, OperationOutcomeIssue, Resource,
    },
    terminology::{IssueSeverity, IssueType},
    types::{FHIRString, Reference},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_ops::OperationExecutor;
use haste_jwt::{ProjectId, ResourceId, TenantId, UserRole};

fn derive_user_id(
    user_reference: Option<Reference>,
) -> Result<Option<String>, OperationOutcomeError> {
    if let Some(reference_string) = user_reference
        .and_then(|u| u.reference)
        .and_then(|r| r.value)
    {
        let reference_chunks = reference_string.split('/').collect::<Vec<_>>();
        let [resource_type, resource_id] = reference_chunks.as_slice() else {
            return Err(OperationOutcomeError::error(
                IssueType::invalid(),
                "Invalid user reference format".to_string(),
            ));
        };

        if *resource_type != "User" {
            return Err(OperationOutcomeError::error(
                IssueType::invalid(),
                "User reference must refer to a User resource".to_string(),
            ));
        }

        return Ok(Some(resource_id.to_string()));
    }

    Ok(None)
}

/// The user the policy is evaluated for. Only an admin or owner may name another
/// user; anyone else is evaluated as themselves, so the operation cannot be used
/// as an allow/deny oracle for other subjects.
fn principal_user_id<B: Backend>(
    ctx: &ServerCTX<B>,
    user: Option<Reference>,
) -> Result<String, OperationOutcomeError> {
    let caller = ctx.user.claims.user_id.as_ref().to_string();
    if matches!(ctx.user.claims.user_role, UserRole::Admin | UserRole::Owner) {
        Ok(derive_user_id(user)?.unwrap_or(caller))
    } else {
        Ok(caller)
    }
}

/// The operation's answer: one informational issue carrying `message`.
fn outcome(message: String) -> HasteHealthEvaluatePolicy::Output {
    HasteHealthEvaluatePolicy::Output {
        return_: OperationOutcome {
            issue: vec![OperationOutcomeIssue {
                severity: IssueSeverity::information(),
                code: IssueType::informational(),
                diagnostics: Some(Box::new(FHIRString {
                    value: Some(message),
                    ..Default::default()
                })),
                ..Default::default()
            }],
            ..Default::default()
        },
    }
}

async fn evaluate_v2<B: Backend>(
    context: &ServerOperationContext<B>,
    tenant: TenantId,
    project: ProjectId,
    policy: AccessPolicyV2,
    request: FHIRRequest,
    user_id: String,
) -> Result<String, OperationOutcomeError> {
    // Use System context for policy evaluation.
    let system_ctx = Arc::new(ServerCTX::system(
        tenant.clone(),
        project.clone(),
        context.ctx.client.clone(),
        context.ctx.rate_limit.clone(),
    ));
    let environment =
        PolicyEnvironment::new(tenant, project, request, Arc::new(UserInfo { id: user_id }));
    let result = haste_access_control::evaluate_policy(
        Arc::new(PolicyContext::new(
            context.ctx.client.clone(),
            system_ctx,
            environment,
        )),
        Arc::new(policy),
    )
    .await?;

    Ok(match result {
        PermissionLevel::Allow => "Policy approved user access.",
        _ => "Policy denied user access.",
    }
    .to_string())
}

/// The policy's decision and, on a permit, the obligations that come with it.
async fn evaluate_v3(policy: &AccessPolicyV3, request: FHIRRequest, user_id: String) -> String {
    let environment = v3::Environment {
        request: RequestReflection::new(request, Channel::Rest),
        principal: v3::Principal { user: user_id },
    };
    let (decision, obligations) = v3::evaluate(&environment, policy).await;

    let mut message = format!("Policy decision: {decision}.");
    if !obligations.is_empty() {
        let obligations = obligations
            .iter()
            .map(|obligation| {
                let code = obligation.code.as_str().unwrap_or_default();
                match obligation.value.as_ref().and_then(|v| v.value.as_deref()) {
                    Some(value) => format!("{code}={value}"),
                    None => code.to_string(),
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        message = format!("{message} Obligations: {obligations}.");
    }
    message
}

pub fn evaluate_policy_op<B: Backend>() -> OperationExecutor<
    ServerOperationContext<B>,
    HasteHealthEvaluatePolicy::Input,
    HasteHealthEvaluatePolicy::Output,
> {
    OperationExecutor::new(
        &HasteHealthEvaluatePolicy::OPERATION_DEFINITION,
        Box::new(
            |context: ServerOperationContext<B>,
             tenant: TenantId,
             project: ProjectId,
             request: &InvocationRequest,
             input: HasteHealthEvaluatePolicy::Input| {
                let request = request.clone();

                Box::pin(async move {
                    let InvocationRequest::Instance(invocation) = &request else {
                        return Err(OperationOutcomeError::fatal(
                            IssueType::exception(),
                            "EvaluatePolicy operation only supported at Instance level".to_string(),
                        ));
                    };

                    let Some(entry) = input.request.entry.as_ref().and_then(|e| e.first()) else {
                        return Err(OperationOutcomeError::fatal(
                            IssueType::invalid(),
                            "EvaluatePolicy operation requires a request entry".to_string(),
                        ));
                    };
                    let fhir_request =
                        bundle_entry_to_fhir_request(entry.clone(), &context.ctx).await?;
                    let user_id = principal_user_id(&context.ctx, input.user)?;

                    let policy = context
                        .state
                        .repo
                        .read_latest(
                            &tenant,
                            &project,
                            &invocation.resource_type,
                            &ResourceId::new(invocation.id.clone()),
                        )
                        .await?;

                    let message = match policy {
                        Some(Resource::AccessPolicyV2(policy)) => {
                            evaluate_v2(&context, tenant, project, policy, fhir_request, user_id)
                                .await?
                        }
                        Some(Resource::AccessPolicyV3(policy)) => {
                            evaluate_v3(&policy, fhir_request, user_id).await
                        }
                        Some(_) => {
                            return Err(OperationOutcomeError::fatal(
                                IssueType::invalid(),
                                "EvaluatePolicy operation must be invoked on an AccessPolicyV2 or AccessPolicyV3 resource".to_string(),
                            ));
                        }
                        None => {
                            return Err(OperationOutcomeError::fatal(
                                IssueType::not_found(),
                                format!(
                                    "{} resource with id '{}' not found",
                                    invocation.resource_type.as_ref(),
                                    invocation.id
                                ),
                            ));
                        }
                    };

                    Ok(outcome(message))
                })
            },
        ),
    )
}
