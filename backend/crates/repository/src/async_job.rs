//! Storage for asynchronous jobs: work a request kicks off and a worker
//! finishes later, followed by the client through the FHIR Asynchronous
//! Request Pattern.
//!
//! A job is one row in `async_jobs`. The server creates it (`accepted`), a
//! worker claims it (`in-progress`) and holds it under a lease it renews while
//! it works, then records the result (`completed` / `failed`). The client can
//! cancel it at any point (`cancelled`), and every job expires.
//!
//! See the `20261014120000_async_jobs` migration for why this is its own table
//! rather than a kind on the indexing lock table.

use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId, claims::UserTokenClaims};
use serde::{Deserialize, Serialize};
use sqlx::types::{Json, time::OffsetDateTime};
use std::time::Duration;

/// Where a job is in its life. Mirrors the `async_job_status` enum.
#[derive(Clone, Copy, Debug, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "async_job_status", rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum AsyncJobStatus {
    Accepted,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

impl AsyncJobStatus {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            AsyncJobStatus::Accepted => "accepted",
            AsyncJobStatus::InProgress => "in-progress",
            AsyncJobStatus::Completed => "completed",
            AsyncJobStatus::Failed => "failed",
            AsyncJobStatus::Cancelled => "cancelled",
        }
    }

    /// Whether the job may still run: queued, or claimed and running.
    #[must_use]
    pub fn is_live(&self) -> bool {
        matches!(self, AsyncJobStatus::Accepted | AsyncJobStatus::InProgress)
    }
}

impl std::fmt::Display for AsyncJobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A job as stored.
#[derive(sqlx::FromRow, Clone, Debug)]
pub struct AsyncJob {
    pub id: String,
    pub tenant: TenantId,
    pub project: ProjectId,
    pub kind: String,
    pub status: AsyncJobStatus,
    pub request: String,
    pub parameters: Json<serde_json::Value>,
    /// The submitter's verified token claims; the worker runs the job as them.
    pub claims: Json<UserTokenClaims>,
    pub submitted_by: String,
    pub progress: Option<String>,
    pub output: Option<Json<serde_json::Value>>,
    pub output_content_type: Option<String>,
    /// An `OperationOutcome`, once failed.
    pub error: Option<Json<serde_json::Value>>,
    pub attempts: i32,
    pub max_attempts: i32,
    pub locked_by: Option<String>,
    pub lease_expires_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub started_at: Option<OffsetDateTime>,
    pub finished_at: Option<OffsetDateTime>,
    pub expires_at: OffsetDateTime,
}

/// What a kick-off records about a new job.
#[derive(Debug)]
pub struct CreateAsyncJob {
    pub kind: String,
    pub request: String,
    pub parameters: serde_json::Value,
    pub claims: UserTokenClaims,
    /// How long the job and its result are kept, from now and again from
    /// completion.
    pub retention: Duration,
    pub max_attempts: i32,
}

/// Identifies the worker holding a job, and for how long one renewal lasts.
#[derive(Clone, Debug)]
pub struct JobLease {
    pub worker: String,
    pub duration: Duration,
}

/// What a lease renewal found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeaseRenewal {
    /// Still ours; keep going.
    Renewed,
    /// The job is no longer ours to run: it was cancelled, or its lease lapsed
    /// and another worker took it. Stop working on it.
    Lost,
}

/// How a finished job ended.
#[derive(Clone, Debug)]
pub enum JobResult {
    Completed {
        output: serde_json::Value,
        content_type: String,
    },
    Failed {
        /// An `OperationOutcome` resource as JSON.
        outcome: serde_json::Value,
    },
}

pub trait AsyncJobProvider {
    /// Queues a job. Its id is generated here and is what the status URL
    /// carries.
    fn create_job(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        job: CreateAsyncJob,
    ) -> impl Future<Output = Result<AsyncJob, OperationOutcomeError>> + Send;

    /// A job of this project, whatever its state, until it expires.
    fn read_job(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        id: &str,
    ) -> impl Future<Output = Result<Option<AsyncJob>, OperationOutcomeError>> + Send;

    /// Jobs of this project that are queued or running. Kick-offs cap this.
    fn count_live_jobs(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
    ) -> impl Future<Output = Result<i64, OperationOutcomeError>> + Send;

    /// Cancels a job of this project. A queued job never runs; a running one
    /// finds out at its next lease renewal; a finished one gives up its
    /// result. Returns the job as it was before, or `None` if there is no
    /// such job or it was already cancelled.
    fn cancel_job(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        id: &str,
    ) -> impl Future<Output = Result<Option<AsyncJob>, OperationOutcomeError>> + Send;

    /// Takes the oldest claimable job of one of `kinds` for `lease.worker`:
    /// queued, or in progress with a lapsed lease, and with attempts left.
    /// Claiming is one statement, so two workers never take the same job.
    fn claim_job(
        &self,
        kinds: &[&str],
        lease: &JobLease,
    ) -> impl Future<Output = Result<Option<AsyncJob>, OperationOutcomeError>> + Send;

    /// Extends the lease on a job this worker holds and records `progress`
    /// if given. `Lost` means stop: the job was cancelled or taken over.
    fn renew_lease(
        &self,
        id: &str,
        lease: &JobLease,
        progress: Option<&str>,
    ) -> impl Future<Output = Result<LeaseRenewal, OperationOutcomeError>> + Send;

    /// Records the result of a job this worker holds and releases the lease.
    /// Returns false if the job was no longer this worker's to finish (it
    /// was cancelled or taken over meanwhile), in which case the result is
    /// dropped.
    fn finish_job(
        &self,
        id: &str,
        worker: &str,
        result: JobResult,
        retention: Duration,
    ) -> impl Future<Output = Result<bool, OperationOutcomeError>> + Send;

    /// Fails every in-progress job whose lease has lapsed and that has no
    /// attempts left, recording `outcome` on each. Returns how many.
    fn fail_abandoned_jobs(
        &self,
        outcome: &serde_json::Value,
        retention: Duration,
    ) -> impl Future<Output = Result<u64, OperationOutcomeError>> + Send;

    /// Removes every job past its `expires_at`, returning them so their
    /// handlers can remove what they produced.
    fn delete_expired_jobs(
        &self,
        limit: i64,
    ) -> impl Future<Output = Result<Vec<AsyncJob>, OperationOutcomeError>> + Send;
}
