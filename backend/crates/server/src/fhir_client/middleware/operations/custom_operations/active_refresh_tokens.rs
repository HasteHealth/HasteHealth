use crate::fhir_client::middleware::{Backend, operations::ServerOperationContext};
use haste_fhir_client::request::InvocationRequest;
use haste_fhir_generated_ops::generated::HasteHealthListRefreshTokens;
use haste_fhir_model::r4::{
    datetime::parse_datetime,
    generated::types::{FHIRDateTime, FHIRId, FHIRString},
};
use haste_fhir_ops::OperationExecutor;
use haste_jwt::{ProjectId, TenantId};
use haste_repository::{
    admin::ProjectModelAdmin,
    types::authorization_code::{AuthorizationCodeKind, AuthorizationCodeSearchClaims},
};
use sqlx::types::time::OffsetDateTime;
use tower_sessions::cookie::time::format_description;

fn format_datetime(datetime: &OffsetDateTime) -> Option<String> {
    datetime
        .format(
            &format_description::parse_borrowed::<2>(
                "[year]-[month]-[day]T[hour]:[minute]:[second][offset_hour \
         sign:mandatory]:[offset_minute]",
            )
            .expect("failed to create formatter"),
        )
        .ok()
}

pub fn active_refresh_tokens_op<B: Backend>() -> OperationExecutor<
    ServerOperationContext<B>,
    HasteHealthListRefreshTokens::Input,
    HasteHealthListRefreshTokens::Output,
> {
    OperationExecutor::new(
        &HasteHealthListRefreshTokens::OPERATION_DEFINITION,
        Box::new(
            |context: ServerOperationContext<B>,
             tenant: TenantId,
             project: ProjectId,
             _request: &InvocationRequest,
             _input: HasteHealthListRefreshTokens::Input| {
                Box::pin(async move {
                    let active_refresh_tokens = ProjectModelAdmin::search(
                        context.state.repo.as_ref(),
                        &tenant,
                        &project,
                        &AuthorizationCodeSearchClaims {
                            client_id: None,
                            code: None,
                            kind: Some(AuthorizationCodeKind::RefreshToken),
                            user_id: Some(context.ctx.user.claims.sub.as_ref().to_string()),
                            user_agent: None,
                            is_expired: Some(false),
                        },
                    )
                    .await?;

                    Ok(HasteHealthListRefreshTokens::Output {
                        refresh_tokens: Some(
                            active_refresh_tokens
                                .into_iter()
                                .map(|token| HasteHealthListRefreshTokens::OutputRefreshTokens {
                                    client_id: FHIRId {
                                        value: token.client_id,
                                        ..Default::default()
                                    },
                                    user_agent: FHIRString {
                                        value: token
                                            .meta
                                            .as_ref()
                                            .and_then(|meta| meta.get("user_agent"))
                                            .and_then(|ua| ua.as_str())
                                            .map(|s| s.to_string()),
                                        ..Default::default()
                                    },
                                    created_at: FHIRDateTime {
                                        value: token
                                            .created_at
                                            .and_then(|dt| format_datetime(&dt))
                                            .and_then(|dt| parse_datetime(&dt).ok()),
                                        ..Default::default()
                                    },
                                })
                                .collect(),
                        ),
                    })
                })
            },
        ),
    )
}
