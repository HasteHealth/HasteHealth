use crate::fhir_client::middleware::{Backend, operations::ServerOperationContext};
use haste_fhir_client::request::InvocationRequest;
use haste_fhir_generated_ops::generated::HasteHealthDeleteRefreshToken;
use haste_fhir_model::r4::generated::{
    resources::{OperationOutcome, OperationOutcomeIssue},
    terminology::{IssueSeverity, IssueType},
    types::FHIRString,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_ops::OperationExecutor;
use haste_jwt::{ProjectId, TenantId};
use haste_repository::{
    admin::ProjectModelAdmin,
    types::authorization_code::{
        AuthorizationCode, AuthorizationCodeKind, AuthorizationCodeSearchClaims,
    },
};

pub fn delete_refresh_token_op<B: Backend>() -> OperationExecutor<
    ServerOperationContext<B>,
    HasteHealthDeleteRefreshToken::Input,
    HasteHealthDeleteRefreshToken::Output,
> {
    OperationExecutor::new(
        &HasteHealthDeleteRefreshToken::OPERATION_DEFINITION,
        Box::new(
            |context: ServerOperationContext<B>,
             tenant: TenantId,
             project: ProjectId,
             _request: &InvocationRequest,
             input: HasteHealthDeleteRefreshToken::Input| {
                Box::pin(async move {
                    let client_id = input.client_id.value.ok_or_else(|| {
                        OperationOutcomeError::error(
                            IssueType::exception(),
                            "Must provide client_id".to_string(),
                        )
                    })?;

                    let user_agent = input.user_agent.and_then(|ua| ua.value).ok_or_else(|| {
                        OperationOutcomeError::error(
                            IssueType::exception(),
                            "Must provide user_agent".to_string(),
                        )
                    })?;

                    let refresh_token = ProjectModelAdmin::search(
                        context.state.repo.as_ref(),
                        &tenant,
                        &project,
                        &AuthorizationCodeSearchClaims {
                            client_id: Some(client_id.clone()),
                            code: None,
                            kind: Some(AuthorizationCodeKind::RefreshToken),
                            user_id: Some(context.ctx.user.claims.sub.as_ref().to_string()),
                            user_agent: Some(user_agent),
                            is_expired: None,
                        },
                    )
                    .await?;

                    let refresh_token = refresh_token.first().ok_or_else(|| {
                        OperationOutcomeError::fatal(
                            IssueType::not_found(),
                            "Refresh token not found".to_string(),
                        )
                    })?;

                    ProjectModelAdmin::<_, AuthorizationCode, _, _, _>::delete(
                        context.state.repo.as_ref(),
                        &tenant,
                        &project,
                        &refresh_token.code,
                    )
                    .await?;

                    Ok(HasteHealthDeleteRefreshToken::Output {
                        return_: OperationOutcome {
                            issue: vec![OperationOutcomeIssue {
                                severity: IssueSeverity::information(),
                                code: IssueType::informational(),
                                diagnostics: Some(Box::new(FHIRString {
                                    value: Some(format!(
                                        "Deleted refresh token for client '{}'",
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
