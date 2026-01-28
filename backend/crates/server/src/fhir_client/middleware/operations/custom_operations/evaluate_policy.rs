use std::sync::Arc;

use crate::fhir_client::{
    ServerCTX, batch_transaction_processing::bundle_entry_to_fhir_request,
    middleware::operations::ServerOperationContext,
};
use haste_access_control::context::{PermissionLevel, PolicyContext, PolicyEnvironment, UserInfo};
use haste_fhir_client::request::InvocationRequest;
use haste_fhir_generated_ops::generated::HasteHealthEvaluatePolicy;

use haste_fhir_model::r4::generated::{
    resources::{OperationOutcome, OperationOutcomeIssue, Resource, ResourceType},
    terminology::{IssueSeverity, IssueType},
    types::{FHIRString, Reference},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_ops::OperationExecutor;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, ResourceId, TenantId};
use haste_repository::Repository;

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
                IssueType::Invalid(None),
                "Invalid user reference format".to_string(),
            ));
        };

        if *resource_type != "User" {
            return Err(OperationOutcomeError::error(
                IssueType::Invalid(None),
                "User reference must refer to a User resource".to_string(),
            ));
        }

        return Ok(Some(resource_id.to_string()));
    }

    Ok(None)
}

pub fn evaluate_policy_op<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>() -> OperationExecutor<
    ServerOperationContext<Repo, Search, Terminology>,
    HasteHealthEvaluatePolicy::Input,
    HasteHealthEvaluatePolicy::Output,
> {
    OperationExecutor::new(
        HasteHealthEvaluatePolicy::CODE.to_string(),
        Box::new(
            |context: ServerOperationContext<Repo, Search, Terminology>,
             tenant: TenantId,
             project: ProjectId,
             request: &InvocationRequest,
             input: HasteHealthEvaluatePolicy::Input| {
                let request = request.clone();

                Box::pin(async move {
                    match &request {
                        InvocationRequest::Instance(invocation_instance) => {
                            // Use System context for policy evaluation.
                            let system_ctx = Arc::new(ServerCTX::system(
                                tenant.clone(),
                                project.clone(),
                                context.ctx.client.clone(),
                                context.ctx.rate_limit.clone(),
                            ));

                            if invocation_instance.resource_type != ResourceType::AccessPolicyV2 {
                                return Err(OperationOutcomeError::fatal(
                                    IssueType::Invalid(None),
                                    "EvaluatePolicy operation must be invoked on an AccessPolicyV2 resource".to_string(),
                                ));
                            }

                            let Some(Resource::AccessPolicyV2(policy)) = context
                                .state
                                .repo
                                .read_latest(
                                    &tenant,
                                    &project,
                                    &ResourceType::AccessPolicyV2,
                                    &ResourceId::new(invocation_instance.id.clone()),
                                )
                                .await?
                            else {
                                return Err(OperationOutcomeError::fatal(
                                    IssueType::NotFound(None),
                                    format!(
                                        "AccessPolicyV2 resource with id '{}' not found",
                                        invocation_instance.id
                                    ),
                                ));
                            };

                            let Some(entry) = input.request.entry.as_ref().and_then(|e| e.get(0))
                            else {
                                return Err(OperationOutcomeError::fatal(
                                    IssueType::Invalid(None),
                                    "EvaluatePolicy operation requires a request entry".to_string(),
                                ));
                            };
                            let fhir_request = bundle_entry_to_fhir_request(entry.clone())?;

                            let result = haste_access_control::evaluate_policy(
                                Arc::new(PolicyContext::new(
                                    context.ctx.client.clone(),
                                    system_ctx,
                                    PolicyEnvironment::new(
                                        tenant.clone(),
                                        project.clone(),
                                        fhir_request,
                                        Arc::new(UserInfo {
                                            id: derive_user_id(input.user)?.unwrap_or(
                                                context.ctx.user.user_id.as_ref().to_string(),
                                            ),
                                        }),
                                    ),
                                )),
                                Arc::new(policy),
                            )
                            .await?;

                            match result {
                                PermissionLevel::Allow => Ok(HasteHealthEvaluatePolicy::Output {
                                    return_: OperationOutcome {
                                        issue: vec![OperationOutcomeIssue {
                                            severity: Box::new(IssueSeverity::Information(None)),
                                            code: Box::new(IssueType::Informational(None)),
                                            diagnostics: Some(Box::new(FHIRString {
                                                value: Some(
                                                    "Policy approved user access.".to_string(),
                                                ),
                                                ..Default::default()
                                            })),
                                            ..Default::default()
                                        }],
                                        ..Default::default()
                                    },
                                }),
                                _ => Ok(HasteHealthEvaluatePolicy::Output {
                                    return_: OperationOutcome {
                                        issue: vec![OperationOutcomeIssue {
                                            severity: Box::new(IssueSeverity::Information(None)),
                                            code: Box::new(IssueType::Informational(None)),
                                            diagnostics: Some(Box::new(FHIRString {
                                                value: Some(
                                                    "Policy denied user access.".to_string(),
                                                ),
                                                ..Default::default()
                                            })),
                                            ..Default::default()
                                        }],
                                        ..Default::default()
                                    },
                                }),
                            }
                        }
                        _ => Err(OperationOutcomeError::fatal(
                            IssueType::Exception(None),
                            "EvaluatePolicy operation only supported at Instance level".to_string(),
                        )),
                    }
                })
            },
        ),
    )
}
