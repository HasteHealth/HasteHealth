use crate::fhir_client::{
    ServerCTX,
    middleware::{ServerMiddlewareState, operations::ServerOperationContext},
};
use haste_fhir_client::{FHIRClient, request::InvocationRequest};
use haste_fhir_generated_ops::generated::HasteHealthIndexingErrors;
use haste_fhir_model::r4::{
    datetime::parse_datetime,
    generated::types::{FHIRCode, FHIRDateTime, FHIRId, FHIRInteger, FHIRString},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_ops::OperationExecutor;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, TenantId};
use haste_repository::{Repository, failed_indexing::FailedIndexingProvider};
use sqlx::types::time::OffsetDateTime;
use std::sync::Arc;
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

fn to_fhir_datetime(datetime: &OffsetDateTime) -> FHIRDateTime {
    FHIRDateTime {
        value: format_datetime(datetime).and_then(|dt| parse_datetime(&dt).ok()),
        ..Default::default()
    }
}

pub fn indexing_errors_op<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>() -> OperationExecutor<
    ServerOperationContext<ServerMiddlewareState<Repo, Search, Terminology>, Client>,
    HasteHealthIndexingErrors::Input,
    HasteHealthIndexingErrors::Output,
> {
    OperationExecutor::new(
        HasteHealthIndexingErrors::CODE.to_string(),
        Box::new(
            |context: ServerOperationContext<
                ServerMiddlewareState<Repo, Search, Terminology>,
                Client,
            >,
             tenant: TenantId,
             project: ProjectId,
             _request: &InvocationRequest,
             _input: HasteHealthIndexingErrors::Input| {
                Box::pin(async move {
                    let failures = FailedIndexingProvider::search(
                        context.state.repo.as_ref(),
                        &tenant,
                        &project,
                    )
                    .await?;

                    Ok(HasteHealthIndexingErrors::Output {
                        errors: Some(
                            failures
                                .into_iter()
                                .map(|entry| HasteHealthIndexingErrors::OutputErrors {
                                    version_id: FHIRId {
                                        value: Some(entry.version_id),
                                        ..Default::default()
                                    },
                                    resource_type: FHIRCode {
                                        value: Some(entry.resource_type),
                                        ..Default::default()
                                    },
                                    fhir_method: FHIRCode {
                                        value: Some(entry.fhir_method.as_str().to_string()),
                                        ..Default::default()
                                    },
                                    sequence: FHIRInteger {
                                        value: Some(entry.sequence),
                                        ..Default::default()
                                    },
                                    attempt_count: FHIRInteger {
                                        value: Some(i64::from(entry.attempt_count)),
                                        ..Default::default()
                                    },
                                    error_message: FHIRString {
                                        value: Some(entry.error_message),
                                        ..Default::default()
                                    },
                                    first_failed_at: to_fhir_datetime(&entry.first_failed_at),
                                    last_failed_at: to_fhir_datetime(&entry.last_failed_at),
                                    resolved_at: entry.resolved_at.as_ref().map(to_fhir_datetime),
                                })
                                .collect(),
                        ),
                    })
                })
            },
        ),
    )
}
