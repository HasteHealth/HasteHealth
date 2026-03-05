use crate::fhir_client::{
    ServerCTX,
    middleware::{ServerMiddlewareState, operations::ServerOperationContext},
};
use haste_fhir_client::{FHIRClient, request::InvocationRequest};
use haste_fhir_generated_ops::generated::HasteHealthDeleteScope;
use haste_fhir_model::r4::generated::{
    resources::{OperationOutcome, OperationOutcomeIssue},
    terminology::{IssueSeverity, IssueType},
    types::FHIRString,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_ops::OperationExecutor;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, TenantId};
use haste_repository::{
    Repository,
    admin::ProjectAuthAdmin,
    types::scope::{ClientId, ScopeKey, UserId},
};
use std::sync::Arc;

pub fn delete_approved_scope_op<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>() -> OperationExecutor<
    ServerOperationContext<ServerMiddlewareState<Repo, Search, Terminology>, Client>,
    HasteHealthDeleteScope::Input,
    HasteHealthDeleteScope::Output,
> {
    OperationExecutor::new(
        HasteHealthDeleteScope::CODE.to_string(),
        Box::new(
            |context: ServerOperationContext<
                ServerMiddlewareState<Repo, Search, Terminology>,
                Client,
            >,
             tenant: TenantId,
             project: ProjectId,
             _request: &InvocationRequest,
             input: HasteHealthDeleteScope::Input| {
                Box::pin(async move {
                    let client_id = input.client_id.value.ok_or_else(|| {
                        OperationOutcomeError::error(
                            IssueType::Exception(None),
                            "Must provide client_id".to_string(),
                        )
                    })?;

                    ProjectAuthAdmin::delete(
                        context.state.repo.as_ref(),
                        &tenant,
                        &project,
                        &ScopeKey(
                            ClientId::new(client_id.clone()),
                            UserId::new(context.ctx.user.sub.as_ref().to_string()),
                        ),
                    )
                    .await?;

                    Ok(HasteHealthDeleteScope::Output {
                        return_: OperationOutcome {
                            issue: vec![OperationOutcomeIssue {
                                severity: Box::new(IssueSeverity::Information(None)),
                                code: Box::new(IssueType::Informational(None)),
                                diagnostics: Some(Box::new(FHIRString {
                                    value: Some(format!(
                                        "Deleted approved scope for client '{}'",
                                        client_id
                                    )),
                                    ..Default::default()
                                })),

                                ..Default::default()
                            }],
                            ..Default::default()
                        },
                    })
                })
            },
        ),
    )
}
