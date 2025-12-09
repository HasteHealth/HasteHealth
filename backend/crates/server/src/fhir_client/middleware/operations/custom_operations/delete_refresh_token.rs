use crate::fhir_client::middleware::operations::ServerOperationContext;
use haste_fhir_client::request::InvocationRequest;
use haste_fhir_generated_ops::generated::HasteHealthDeleteRefreshToken;
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
    types::authorization_code::{
        AuthorizationCode, AuthorizationCodeKind, AuthorizationCodeSearchClaims,
    },
};

pub fn delete_refresh_token<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>() -> OperationExecutor<
    ServerOperationContext<Repo, Search, Terminology>,
    HasteHealthDeleteRefreshToken::Input,
    HasteHealthDeleteRefreshToken::Output,
> {
    OperationExecutor::new(
        HasteHealthDeleteRefreshToken::CODE.to_string(),
        Box::new(
            |context: ServerOperationContext<Repo, Search, Terminology>,
             tenant: TenantId,
             project: ProjectId,
             _request: &InvocationRequest,
             input: HasteHealthDeleteRefreshToken::Input| {
                Box::pin(async move {
                    let client_id = input.client_id.value.ok_or_else(|| {
                        OperationOutcomeError::error(
                            IssueType::Exception(None),
                            "Must provide client_id".to_string(),
                        )
                    })?;

                    let user_agent = input.user_agent.and_then(|ua| ua.value).ok_or_else(|| {
                        OperationOutcomeError::error(
                            IssueType::Exception(None),
                            "Must provide user_agent".to_string(),
                        )
                    })?;

                    let refresh_token = ProjectAuthAdmin::search(
                        context.state.repo.as_ref(),
                        &tenant,
                        &project,
                        &AuthorizationCodeSearchClaims {
                            client_id: Some(client_id.clone()),
                            code: None,
                            kind: Some(AuthorizationCodeKind::RefreshToken),
                            user_id: Some(context.ctx.user.sub.as_ref().to_string()),
                            user_agent: Some(user_agent),
                            is_expired: None,
                        },
                    )
                    .await?;

                    let refresh_token = refresh_token.get(0).ok_or_else(|| {
                        OperationOutcomeError::fatal(
                            IssueType::NotFound(None),
                            "Refresh token not found".to_string(),
                        )
                    })?;

                    ProjectAuthAdmin::<_, AuthorizationCode, _, _, _>::delete(
                        context.state.repo.as_ref(),
                        &tenant,
                        &project,
                        &refresh_token.code,
                    )
                    .await?;

                    Ok(HasteHealthDeleteRefreshToken::Output {
                        return_: OperationOutcome {
                            issue: vec![OperationOutcomeIssue {
                                severity: Box::new(IssueSeverity::Information(None)),
                                code: Box::new(IssueType::Informational(None)),
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
