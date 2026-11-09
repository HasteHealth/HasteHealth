use crate::{
    client_assertion::ClientAssertionReplayProvider,
    pg::{PGConnection, StoreError},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId};
use sqlx::{Acquire, PgExecutor};
use std::time::Duration;

/// Drops the records of assertions that can no longer be replayed. Cheap: it
/// walks the `expires_at` index, and there is one row per assertion accepted
/// in the last few minutes.
async fn purge_expired<'e, E>(executor: E) -> Result<(), OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    sqlx::query("DELETE FROM client_assertion_jti WHERE expires_at < NOW()")
        .execute(executor)
        .await
        .map_err(StoreError::from)?;

    Ok(())
}

/// Inserts the `jti`; the primary key refuses one already recorded, which
/// answers `false`.
async fn record<'e, E>(
    executor: E,
    tenant: &TenantId,
    project: &ProjectId,
    client_id: &str,
    jti: &str,
    lifetime: Duration,
) -> Result<bool, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let lifetime_seconds = i64::try_from(lifetime.as_secs()).unwrap_or(i64::MAX);

    let result = sqlx::query(
        r"
            INSERT INTO client_assertion_jti (tenant, project, client_id, jti, expires_at)
            VALUES ($1, $2, $3, $4, NOW() + make_interval(secs => $5::double precision))
            ON CONFLICT DO NOTHING
        ",
    )
    .bind(tenant.as_ref())
    .bind(project.as_ref())
    .bind(client_id)
    .bind(jti)
    .bind(lifetime_seconds)
    .execute(executor)
    .await
    .map_err(StoreError::from)?;

    Ok(result.rows_affected() == 1)
}

impl ClientAssertionReplayProvider for PGConnection {
    async fn record_client_assertion_jti(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        client_id: &str,
        jti: &str,
        lifetime: Duration,
    ) -> Result<bool, OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => {
                purge_expired(pool).await?;
                record(pool, tenant, project, client_id, jti, lifetime).await
            }
            PGConnection::Transaction(tx, _, _) => {
                let mut tx = tx.lock().await;
                let conn = (&mut (*tx)).acquire().await.map_err(StoreError::from)?;
                purge_expired(&mut *conn).await?;
                record(&mut *conn, tenant, project, client_id, jti, lifetime).await
            }
        }
    }
}
