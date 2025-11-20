use crate::fhir_client::middleware::operations::ServerOperationContext;
use haste_fhir_generated_ops::generated::HasteHealthListScopes;
use haste_fhir_model::r4::{
    datetime::parse_datetime,
    generated::types::{FHIRDateTime, FHIRId, FHIRString},
};
use haste_fhir_ops::OperationExecutor;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, TenantId};
use haste_repository::{
    Repository,
    admin::ProjectAuthAdmin,
    types::scope::{ScopeSearchClaims, UserId},
};
use sqlx::types::time::OffsetDateTime;
use tower_sessions::cookie::time::format_description;

fn format_datetime(datetime: &OffsetDateTime) -> Option<String> {
    let res = datetime
        .format(
            &format_description::parse(
                "[year]-[month]-[day]T[hour]:[minute]:[second][offset_hour \
         sign:mandatory]:[offset_minute]",
            )
            .expect("failed to create formatter"),
        )
        .ok();
    res
}

pub fn approved_scopes<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>() -> OperationExecutor<
    ServerOperationContext<Repo, Search, Terminology>,
    HasteHealthListScopes::Input,
    HasteHealthListScopes::Output,
> {
    OperationExecutor::new(
        HasteHealthListScopes::CODE.to_string(),
        Box::new(
            |context: ServerOperationContext<Repo, Search, Terminology>,
             tenant: TenantId,
             project: ProjectId,
             _input: HasteHealthListScopes::Input| {
                Box::pin(async move {
                    let active_scopes = ProjectAuthAdmin::search(
                        context.state.repo.as_ref(),
                        &tenant,
                        &project,
                        &ScopeSearchClaims {
                            user_: Some(UserId::new(context.ctx.user.sub.as_ref().to_string())),
                            client: None,
                        },
                    )
                    .await?;

                    Ok(HasteHealthListScopes::Output {
                        scopes: Some(
                            active_scopes
                                .into_iter()
                                .map(|scope| HasteHealthListScopes::OutputScopes {
                                    client_id: FHIRId {
                                        value: Some(scope.client),
                                        ..Default::default()
                                    },
                                    scopes: FHIRString {
                                        value: Some(scope.scope.into()),
                                        ..Default::default()
                                    },
                                    created_at: FHIRDateTime {
                                        value: format_datetime(&scope.created_at)
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
