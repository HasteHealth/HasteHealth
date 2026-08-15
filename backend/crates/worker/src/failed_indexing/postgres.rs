use crate::failed_indexing::{FailedIndexRecord, FailedIndexingProvider};
use haste_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use haste_repository::pg::PGConnection;
use sqlx::{Acquire, Postgres, QueryBuilder};

#[derive(OperationOutcomeError, Debug)]
pub enum FailedIndexingError {
    #[fatal(code = "exception", diagnostic = "SQL error occurred: '{arg0}'")]
    SQLError(#[from] sqlx::Error),
    #[fatal(
        code = "exception",
        diagnostic = "Recording indexing failures requires a transaction."
    )]
    InvalidConnection,
}

impl FailedIndexingProvider for PGConnection {
    async fn record_failures(
        &self,
        failures: &[FailedIndexRecord],
    ) -> Result<(), OperationOutcomeError> {
        if failures.is_empty() {
            return Ok(());
        }

        match self {
            PGConnection::Transaction(tx, _, _) => {
                let mut tx = tx.lock().await;
                let conn = (&mut (*tx))
                    .acquire()
                    .await
                    .map_err(FailedIndexingError::from)?;

                let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
                    "INSERT INTO failed_search_indexing \
                     (tenant, project, version_id, resource_type, fhir_method, error_message) ",
                );

                query_builder.push_values(failures, |mut b, failure| {
                    b.push_bind(failure.tenant.as_ref())
                        .push_bind(failure.project.as_ref())
                        .push_bind(failure.version_id.as_ref())
                        .push_bind(&failure.resource_type)
                        .push_bind(failure.fhir_method.clone())
                        .push_bind(&failure.error_message);
                });

                query_builder.push(
                    r" ON CONFLICT (tenant, project, version_id) DO UPDATE SET
                      attempt_count = failed_search_indexing.attempt_count + 1,
                      last_failed_at = now(),
                      error_message = EXCLUDED.error_message,
                      resolved_at = NULL",
                );

                query_builder
                    .build()
                    .execute(conn)
                    .await
                    .map_err(FailedIndexingError::from)?;

                Ok(())
            }
            PGConnection::Pool(..) => Err(FailedIndexingError::InvalidConnection.into()),
        }
    }
}
