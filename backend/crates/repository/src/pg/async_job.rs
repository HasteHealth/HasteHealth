//! Postgres storage for [`AsyncJobProvider`]. Every state change is one
//! statement guarded by the state it expects (`status`, `locked_by`), so two
//! workers, or a worker and a cancelling client, can never both win.

use crate::{
    async_job::{
        AsyncJob, AsyncJobProvider, AsyncJobStatus, CreateAsyncJob, JobLease, JobResult,
        LeaseRenewal,
    },
    pg::{PGConnection, StoreError},
    utilities::generate_id,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId};
use sqlx::{PgExecutor, types::Json};
use std::time::Duration;

async fn create_job<'e, E>(
    executor: E,
    tenant: &TenantId,
    project: &ProjectId,
    job: CreateAsyncJob,
) -> Result<AsyncJob, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let submitted_by = job.claims.sub.as_ref().to_string();

    let created = sqlx::query_as::<_, AsyncJob>(
        r"
            INSERT INTO async_jobs
                (id, tenant, project, kind, request, parameters, claims, submitted_by,
                 max_attempts, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, now() + make_interval(secs => $10))
            RETURNING *
        ",
    )
    .bind(generate_id(None))
    .bind(tenant.as_ref())
    .bind(project.as_ref())
    .bind(&job.kind)
    .bind(&job.request)
    .bind(Json(&job.parameters))
    .bind(Json(&job.claims))
    .bind(submitted_by)
    .bind(job.max_attempts)
    .bind(job.retention.as_secs_f64())
    .fetch_one(executor)
    .await
    .map_err(StoreError::from)?;

    Ok(created)
}

async fn read_job<'e, E>(
    executor: E,
    tenant: &TenantId,
    project: &ProjectId,
    id: &str,
) -> Result<Option<AsyncJob>, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let job = sqlx::query_as::<_, AsyncJob>(
        "SELECT * FROM async_jobs WHERE tenant = $1 AND project = $2 AND id = $3",
    )
    .bind(tenant.as_ref())
    .bind(project.as_ref())
    .bind(id)
    .fetch_optional(executor)
    .await
    .map_err(StoreError::from)?;

    Ok(job)
}

async fn count_live_jobs<'e, E>(
    executor: E,
    tenant: &TenantId,
    project: &ProjectId,
) -> Result<i64, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let count: i64 = sqlx::query_scalar(
        r"
            SELECT count(*) FROM async_jobs
            WHERE tenant = $1 AND project = $2 AND status IN ('accepted', 'in-progress')
        ",
    )
    .bind(tenant.as_ref())
    .bind(project.as_ref())
    .fetch_one(executor)
    .await
    .map_err(StoreError::from)?;

    Ok(count)
}

/// Cancelling also expires the job at once, so the next sweep removes it
/// (and whatever its handler produced) instead of waiting out the retention.
async fn cancel_job<'e, E>(
    executor: E,
    tenant: &TenantId,
    project: &ProjectId,
    id: &str,
) -> Result<Option<AsyncJob>, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let job = sqlx::query_as::<_, AsyncJob>(
        r"
            UPDATE async_jobs
            SET status = 'cancelled',
                finished_at = COALESCE(finished_at, now()),
                locked_by = NULL,
                lease_expires_at = NULL,
                expires_at = now()
            WHERE tenant = $1 AND project = $2 AND id = $3 AND status <> 'cancelled'
            RETURNING *
        ",
    )
    .bind(tenant.as_ref())
    .bind(project.as_ref())
    .bind(id)
    .fetch_optional(executor)
    .await
    .map_err(StoreError::from)?;

    Ok(job)
}

/// One statement: the inner select picks and row-locks the job, skipping any
/// another worker is claiming at the same instant, and the update takes it.
/// A retry of an abandoned job starts over, so stale progress is cleared.
async fn claim_job<'e, E>(
    executor: E,
    kinds: &[&str],
    lease: &JobLease,
) -> Result<Option<AsyncJob>, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let kinds: Vec<String> = kinds.iter().map(ToString::to_string).collect();

    let job = sqlx::query_as::<_, AsyncJob>(
        r"
            UPDATE async_jobs
            SET status = 'in-progress',
                locked_by = $1,
                lease_expires_at = now() + make_interval(secs => $2),
                attempts = attempts + 1,
                started_at = COALESCE(started_at, now()),
                progress = NULL
            WHERE id = (
                SELECT id FROM async_jobs
                WHERE kind = ANY($3)
                  AND attempts < max_attempts
                  AND (status = 'accepted'
                       OR (status = 'in-progress' AND lease_expires_at < now()))
                ORDER BY created_at
                LIMIT 1
                FOR NO KEY UPDATE SKIP LOCKED
            )
            RETURNING *
        ",
    )
    .bind(&lease.worker)
    .bind(lease.duration.as_secs_f64())
    .bind(&kinds)
    .fetch_optional(executor)
    .await
    .map_err(StoreError::from)?;

    Ok(job)
}

async fn renew_lease<'e, E>(
    executor: E,
    id: &str,
    lease: &JobLease,
    progress: Option<&str>,
) -> Result<LeaseRenewal, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let result = sqlx::query(
        r"
            UPDATE async_jobs
            SET lease_expires_at = now() + make_interval(secs => $2),
                progress = COALESCE($3, progress)
            WHERE id = $1 AND locked_by = $4 AND status = 'in-progress'
        ",
    )
    .bind(id)
    .bind(lease.duration.as_secs_f64())
    .bind(progress)
    .bind(&lease.worker)
    .execute(executor)
    .await
    .map_err(StoreError::from)?;

    Ok(if result.rows_affected() == 1 {
        LeaseRenewal::Renewed
    } else {
        LeaseRenewal::Lost
    })
}

async fn finish_job<'e, E>(
    executor: E,
    id: &str,
    worker: &str,
    result: JobResult,
    retention: Duration,
) -> Result<bool, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let (status, output, content_type, error) = match result {
        JobResult::Completed {
            output,
            content_type,
        } => (
            AsyncJobStatus::Completed,
            Some(Json(output)),
            Some(content_type),
            None,
        ),
        JobResult::Failed { outcome } => (AsyncJobStatus::Failed, None, None, Some(Json(outcome))),
    };

    let result = sqlx::query(
        r"
            UPDATE async_jobs
            SET status = $2,
                output = $3,
                output_content_type = $4,
                error = $5,
                finished_at = now(),
                locked_by = NULL,
                lease_expires_at = NULL,
                progress = NULL,
                expires_at = now() + make_interval(secs => $6)
            WHERE id = $1 AND locked_by = $7 AND status = 'in-progress'
        ",
    )
    .bind(id)
    .bind(status)
    .bind(output)
    .bind(content_type)
    .bind(error)
    .bind(retention.as_secs_f64())
    .bind(worker)
    .execute(executor)
    .await
    .map_err(StoreError::from)?;

    Ok(result.rows_affected() == 1)
}

async fn fail_abandoned_jobs<'e, E>(
    executor: E,
    outcome: &serde_json::Value,
    retention: Duration,
) -> Result<u64, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let result = sqlx::query(
        r"
            UPDATE async_jobs
            SET status = 'failed',
                error = $1,
                finished_at = now(),
                locked_by = NULL,
                lease_expires_at = NULL,
                progress = NULL,
                expires_at = now() + make_interval(secs => $2)
            WHERE status = 'in-progress'
              AND lease_expires_at < now()
              AND attempts >= max_attempts
        ",
    )
    .bind(Json(outcome))
    .bind(retention.as_secs_f64())
    .execute(executor)
    .await
    .map_err(StoreError::from)?;

    Ok(result.rows_affected())
}

/// Bounded and lock-skipping, so several workers sweeping at once each take
/// their own slice and none waits on another.
async fn delete_expired_jobs<'e, E>(
    executor: E,
    limit: i64,
) -> Result<Vec<AsyncJob>, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let jobs = sqlx::query_as::<_, AsyncJob>(
        r"
            DELETE FROM async_jobs
            WHERE id IN (
                SELECT id FROM async_jobs
                WHERE expires_at < now()
                ORDER BY expires_at
                LIMIT $1
                FOR UPDATE SKIP LOCKED
            )
            RETURNING *
        ",
    )
    .bind(limit)
    .fetch_all(executor)
    .await
    .map_err(StoreError::from)?;

    Ok(jobs)
}

/// Runs `$call` against whichever connection this is. A job statement is a
/// unit on its own, so none of them needs the caller's transaction, but
/// inside one they take part in it like everything else.
macro_rules! with_executor {
    ($self:expr, |$executor:ident| $call:expr) => {
        match $self {
            PGConnection::Pool(pool, _) => {
                let $executor = pool;
                $call.await
            }
            PGConnection::Transaction(tx, _, _) => {
                let mut tx = tx.lock().await;
                let $executor = &mut **tx;
                $call.await
            }
        }
    };
}

impl AsyncJobProvider for PGConnection {
    async fn create_job(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        job: CreateAsyncJob,
    ) -> Result<AsyncJob, OperationOutcomeError> {
        with_executor!(self, |executor| create_job(executor, tenant, project, job))
    }

    async fn read_job(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        id: &str,
    ) -> Result<Option<AsyncJob>, OperationOutcomeError> {
        with_executor!(self, |executor| read_job(executor, tenant, project, id))
    }

    async fn count_live_jobs(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
    ) -> Result<i64, OperationOutcomeError> {
        with_executor!(self, |executor| count_live_jobs(executor, tenant, project))
    }

    async fn cancel_job(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        id: &str,
    ) -> Result<Option<AsyncJob>, OperationOutcomeError> {
        with_executor!(self, |executor| cancel_job(executor, tenant, project, id))
    }

    async fn claim_job(
        &self,
        kinds: &[&str],
        lease: &JobLease,
    ) -> Result<Option<AsyncJob>, OperationOutcomeError> {
        with_executor!(self, |executor| claim_job(executor, kinds, lease))
    }

    async fn renew_lease(
        &self,
        id: &str,
        lease: &JobLease,
        progress: Option<&str>,
    ) -> Result<LeaseRenewal, OperationOutcomeError> {
        with_executor!(self, |executor| renew_lease(executor, id, lease, progress))
    }

    async fn finish_job(
        &self,
        id: &str,
        worker: &str,
        result: JobResult,
        retention: Duration,
    ) -> Result<bool, OperationOutcomeError> {
        with_executor!(self, |executor| finish_job(
            executor, id, worker, result, retention
        ))
    }

    async fn fail_abandoned_jobs(
        &self,
        outcome: &serde_json::Value,
        retention: Duration,
    ) -> Result<u64, OperationOutcomeError> {
        with_executor!(self, |executor| fail_abandoned_jobs(
            executor, outcome, retention
        ))
    }

    async fn delete_expired_jobs(
        &self,
        limit: i64,
    ) -> Result<Vec<AsyncJob>, OperationOutcomeError> {
        with_executor!(self, |executor| delete_expired_jobs(executor, limit))
    }
}
