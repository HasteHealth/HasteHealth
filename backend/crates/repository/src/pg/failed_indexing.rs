use crate::{
    failed_indexing::{FailedIndexEntry, FailedIndexRecord, FailedIndexingProvider},
    pg::{PGConnection, StoreError},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId};
use sqlx::{Acquire, PgExecutor, Postgres, QueryBuilder};

async fn search_failed_indexing<'a, 'e, E>(
    executor: E,
    tenant: &'a TenantId,
    project: &'a ProjectId,
) -> Result<Vec<FailedIndexEntry>, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let entries = sqlx::query_as::<_, FailedIndexEntry>(
        r"
            SELECT r.id, f.version_id, f.resource_type, f.fhir_method, f.attempt_count,
                   f.error_message, f.first_failed_at, f.last_failed_at, f.resolved_at,
                   r.sequence
            FROM failed_search_indexing f
            JOIN resources r
                ON r.tenant = f.tenant AND r.project = f.project AND r.version_id = f.version_id
            WHERE f.tenant = $1 AND f.project = $2
            ORDER BY f.last_failed_at DESC
        ",
    )
    .bind(tenant.as_ref())
    .bind(project.as_ref())
    .fetch_all(executor)
    .await
    .map_err(StoreError::from)?;

    Ok(entries)
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
                let conn = (&mut (*tx)).acquire().await.map_err(StoreError::from)?;

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
                    .map_err(StoreError::from)?;

                Ok(())
            }
            PGConnection::Pool(..) => Err(StoreError::NotTransaction.into()),
        }
    }

    async fn search(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
    ) -> Result<Vec<FailedIndexEntry>, OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => search_failed_indexing(pool, tenant, project).await,
            PGConnection::Transaction(tx, _, _) => {
                let mut tx = tx.lock().await;
                search_failed_indexing(&mut **tx, tenant, project).await
            }
        }
    }
}
