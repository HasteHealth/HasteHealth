//! Runs asynchronous jobs: claims queued rows from `async_jobs`, hands each to
//! the [`JobHandler`] registered for its kind, keeps the job's lease renewed
//! while the handler works, and records the result.
//!
//! Any number of these workers may run at once, on any number of hosts: a
//! claim is one row-locking statement, so no two take the same job, and a
//! worker that dies simply stops renewing, which hands its jobs back to the
//! queue (up to each job's `max_attempts`). The same worker also runs the
//! maintenance sweeps: failing jobs abandoned past their last attempt, and
//! removing expired ones, letting their handlers remove what they produced.

use crate::traits::Worker;
use haste_fhir_model::r4::generated::{resources::Resource, terminology::IssueType};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_repository::{
    async_job::{AsyncJob, AsyncJobProvider, JobLease, JobResult, LeaseRenewal},
    utilities::generate_id,
};
use std::{
    collections::HashMap,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::{
    sync::{Mutex, Semaphore},
    task::JoinHandle,
};

/// What a finished handler hands back, and how the status endpoint serves it.
#[derive(Debug)]
pub enum JobOutput {
    /// Served as `application/fhir+json`: the response the synchronous form
    /// of the request would have had.
    Resource(Box<Resource>),
    /// Served as `application/json`: a Bulk Data manifest, for one.
    Json(serde_json::Value),
}

impl JobOutput {
    fn into_result(self) -> Result<JobResult, OperationOutcomeError> {
        let (output, content_type) = match self {
            JobOutput::Resource(resource) => (
                serde_json::to_value(&resource).map_err(|e| {
                    OperationOutcomeError::fatal(
                        IssueType::exception(),
                        format!("Failed to serialize the job's output: {e}"),
                    )
                })?,
                "application/fhir+json",
            ),
            JobOutput::Json(value) => (value, "application/json"),
        };

        Ok(JobResult::Completed {
            output,
            content_type: content_type.to_string(),
        })
    }
}

/// What a running handler sees of its job, and how it talks back: progress
/// for the client's `X-Progress`, and whether the client has cancelled.
pub struct JobContext {
    pub job: AsyncJob,
    pending_progress: Mutex<Option<String>>,
    cancelled: AtomicBool,
}

impl JobContext {
    fn new(job: AsyncJob) -> Self {
        JobContext {
            job,
            pending_progress: Mutex::new(None),
            cancelled: AtomicBool::new(false),
        }
    }

    /// Records progress to send with the next lease renewal. Only the latest
    /// message is kept.
    pub async fn report(&self, progress: impl Into<String>) {
        *self.pending_progress.lock().await = Some(progress.into());
    }

    /// True once the job is no longer this worker's to run: the client
    /// cancelled it, or the lease lapsed and another worker took it. A
    /// handler should check this between steps and stop; whatever it returns
    /// afterwards is dropped.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }

    async fn take_progress(&self) -> Option<String> {
        self.pending_progress.lock().await.take()
    }
}

pub type HandlerFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, OperationOutcomeError>> + Send + 'a>>;

/// Runs jobs of one kind.
pub trait JobHandler: Send + Sync {
    /// The `kind` of the jobs this handler runs.
    fn kind(&self) -> &'static str;

    /// Runs one job to its result. An `Err` fails the job with that outcome;
    /// a panic fails it too.
    fn run<'a>(&'a self, context: &'a JobContext) -> HandlerFuture<'a, JobOutput>;

    /// Called once for a job that has expired (or was cancelled) and is being
    /// removed, so the handler can remove whatever it produced for it.
    fn expire<'a>(&'a self, job: &'a AsyncJob) -> HandlerFuture<'a, ()> {
        let _ = job;
        Box::pin(async { Ok(()) })
    }
}

#[derive(Clone, Debug)]
pub struct AsyncJobWorkerConfig {
    /// How long to wait before looking for work again after finding none.
    pub poll_interval: Duration,
    /// How long one claim or renewal holds a job. Renewals happen at a third
    /// of this, so a worker has to miss two in a row before its job can be
    /// taken over.
    pub lease_duration: Duration,
    /// Jobs this worker runs at once.
    pub max_concurrent_jobs: usize,
    /// How often the sweeps for abandoned and expired jobs run.
    pub maintenance_interval: Duration,
    /// How long a finished job and its result are kept.
    pub retention: Duration,
}

impl Default for AsyncJobWorkerConfig {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_secs(1),
            lease_duration: Duration::from_secs(60),
            max_concurrent_jobs: 4,
            maintenance_interval: Duration::from_secs(60),
            retention: Duration::from_hours(24),
        }
    }
}

/// Expired rows removed per sweep. Bounds one sweep's work, and the time it
/// holds row locks; what is left waits for the next sweep.
const EXPIRED_SWEEP_LIMIT: i64 = 100;

/// Nothing to poll at all (the claim failed): no reason to look again soon.
const ERROR_BACKOFF: Duration = Duration::from_secs(5);

pub struct AsyncJobWorker<Repo> {
    repo: Arc<Repo>,
    handlers: Arc<HashMap<&'static str, Arc<dyn JobHandler>>>,
    config: AsyncJobWorkerConfig,
    /// Names this worker in `async_jobs.locked_by`.
    worker_id: String,
    running: Arc<Mutex<bool>>,
}

impl<Repo: AsyncJobProvider + Send + Sync + 'static> AsyncJobWorker<Repo> {
    /// A worker that runs the kinds `handlers` cover, and nothing else.
    #[must_use]
    pub fn new(
        repo: Arc<Repo>,
        handlers: Vec<Arc<dyn JobHandler>>,
        config: AsyncJobWorkerConfig,
    ) -> Self {
        let handlers = handlers
            .into_iter()
            .map(|handler| (handler.kind(), handler))
            .collect();

        let host = std::env::var("HOSTNAME").unwrap_or_else(|_| "worker".to_string());
        let worker_id = format!("{host}-{}-{}", std::process::id(), generate_id(Some(6)));

        Self {
            repo,
            handlers: Arc::new(handlers),
            config,
            worker_id,
            running: Arc::new(Mutex::new(true)),
        }
    }

    #[must_use]
    pub fn worker_id(&self) -> &str {
        &self.worker_id
    }
}

/// The outcome recorded on a job whose worker stopped renewing and which has
/// no attempts left.
fn abandoned_outcome() -> serde_json::Value {
    let error = OperationOutcomeError::fatal(
        IssueType::exception(),
        "The job was abandoned: the worker running it stopped responding, and it has no \
         attempts left."
            .to_string(),
    );

    serde_json::to_value(error.outcome()).unwrap_or(serde_json::Value::Null)
}

/// Failing a job after a panic: the panic is logged with the job, the client
/// only sees that it failed.
fn panicked_outcome() -> serde_json::Value {
    let error = OperationOutcomeError::fatal(
        IssueType::exception(),
        "The job failed with an internal error.".to_string(),
    );

    serde_json::to_value(error.outcome()).unwrap_or(serde_json::Value::Null)
}

/// Runs one claimed job: the handler in its own task (so a panic is caught
/// here and fails the job rather than the worker), renewing the lease
/// alongside it, then records the result. `_permit` holds this worker's
/// concurrency slot until the job is done.
async fn run_job<Repo: AsyncJobProvider + Send + Sync + 'static>(
    repo: Arc<Repo>,
    handlers: Arc<HashMap<&'static str, Arc<dyn JobHandler>>>,
    lease: JobLease,
    retention: Duration,
    job: AsyncJob,
    _permit: tokio::sync::OwnedSemaphorePermit,
) {
    let job_id = job.id.clone();
    let kind = job.kind.clone();
    let tenant = job.tenant.clone();
    let project = job.project.clone();
    let attempt = job.attempts;
    let started = Instant::now();

    tracing::info!(
        job = %job_id,
        kind = %kind,
        tenant = %tenant,
        project = %project.as_ref(),
        attempt,
        "Running job"
    );

    let context = Arc::new(JobContext::new(job));

    let renewals = tokio::spawn(renew_lease_until_lost(
        repo.clone(),
        lease.clone(),
        context.clone(),
    ));

    let outcome = match handlers.get(kind.as_str()).cloned() {
        Some(handler) => {
            let handler_context = context.clone();
            match tokio::spawn(async move { handler.run(&handler_context).await }).await {
                Ok(Ok(output)) => output.into_result(),
                Ok(Err(error)) => Ok(JobResult::Failed {
                    outcome: serde_json::to_value(error.outcome())
                        .unwrap_or(serde_json::Value::Null),
                }),
                Err(join_error) => {
                    tracing::error!(job = %job_id, kind = %kind, "Job panicked: {join_error:?}");
                    Ok(JobResult::Failed {
                        outcome: panicked_outcome(),
                    })
                }
            }
        }
        // Only registered kinds are claimed, so this cannot happen; if it
        // does, the job must not sit in-progress until its lease lapses.
        None => Ok(JobResult::Failed {
            outcome: serde_json::to_value(
                OperationOutcomeError::fatal(
                    IssueType::not_supported(),
                    format!("No handler for job kind '{kind}'."),
                )
                .outcome(),
            )
            .unwrap_or(serde_json::Value::Null),
        }),
    };

    renewals.abort();

    if context.is_cancelled() {
        tracing::info!(
            job = %job_id,
            kind = %kind,
            "Job was cancelled or taken over while running; dropping its result"
        );
        return;
    }

    let result = match outcome {
        Ok(result) => result,
        Err(error) => JobResult::Failed {
            outcome: serde_json::to_value(error.outcome()).unwrap_or(serde_json::Value::Null),
        },
    };
    let status = match &result {
        JobResult::Completed { .. } => "completed",
        JobResult::Failed { .. } => "failed",
    };

    match repo
        .finish_job(&job_id, &lease.worker, result, retention)
        .await
    {
        Ok(true) => tracing::info!(
            job = %job_id,
            kind = %kind,
            status,
            elapsed = ?started.elapsed(),
            "Job finished"
        ),
        Ok(false) => tracing::warn!(
            job = %job_id,
            kind = %kind,
            "Job was no longer this worker's to finish; dropping its result"
        ),
        Err(error) => tracing::error!(
            job = %job_id,
            kind = %kind,
            "Failed to record the job's result: {error:?}"
        ),
    }
}

/// Renews the job's lease at a third of its length until the job is lost,
/// carrying any progress the handler reported since the last renewal. A
/// failed renewal (the database was unreachable for a moment) is retried at
/// the next tick; only a definite `Lost` stops the handler.
async fn renew_lease_until_lost<Repo: AsyncJobProvider + Send + Sync + 'static>(
    repo: Arc<Repo>,
    lease: JobLease,
    context: Arc<JobContext>,
) {
    let interval = lease.duration / 3;

    loop {
        tokio::time::sleep(interval).await;

        let progress = context.take_progress().await;
        match repo
            .renew_lease(&context.job.id, &lease, progress.as_deref())
            .await
        {
            Ok(LeaseRenewal::Renewed) => {}
            Ok(LeaseRenewal::Lost) => {
                context.cancelled.store(true, Ordering::Relaxed);
                return;
            }
            Err(error) => tracing::warn!(
                job = %context.job.id,
                "Failed to renew the job's lease, will retry: {error:?}"
            ),
        }
    }
}

/// The sweeps: fail what was abandoned past its last attempt, and remove what
/// has expired, giving each expired job to its handler first.
async fn maintain<Repo: AsyncJobProvider + Send + Sync + 'static>(
    repo: &Repo,
    handlers: &HashMap<&'static str, Arc<dyn JobHandler>>,
    retention: Duration,
) {
    match repo
        .fail_abandoned_jobs(&abandoned_outcome(), retention)
        .await
    {
        Ok(0) => {}
        Ok(failed) => tracing::warn!("Failed {failed} abandoned job(s)"),
        Err(error) => tracing::error!("Failed to sweep abandoned jobs: {error:?}"),
    }

    match repo.delete_expired_jobs(EXPIRED_SWEEP_LIMIT).await {
        Ok(expired) => {
            for job in expired {
                tracing::debug!(job = %job.id, kind = %job.kind, "Removed expired job");
                if let Some(handler) = handlers.get(job.kind.as_str())
                    && let Err(error) = handler.expire(&job).await
                {
                    tracing::error!(
                        job = %job.id,
                        kind = %job.kind,
                        "Handler failed to clean up an expired job: {error:?}"
                    );
                }
            }
        }
        Err(error) => tracing::error!("Failed to sweep expired jobs: {error:?}"),
    }
}

impl<Repo: AsyncJobProvider + Send + Sync + 'static> Worker for AsyncJobWorker<Repo> {
    // Nothing here waits: the loop is spawned and its handle returned.
    fn run(&self) -> impl Future<Output = Result<JoinHandle<()>, OperationOutcomeError>> {
        let repo = self.repo.clone();
        let handlers = self.handlers.clone();
        let config = self.config.clone();
        let running = self.running.clone();
        let lease = JobLease {
            worker: self.worker_id.clone(),
            duration: config.lease_duration,
        };
        let kinds: Vec<&'static str> = handlers.keys().copied().collect();
        let slots = Arc::new(Semaphore::new(config.max_concurrent_jobs.max(1)));

        tracing::info!(
            worker = %self.worker_id,
            kinds = ?kinds,
            "Starting async job worker..."
        );

        let spawned = tokio::spawn(async move {
            // Sweep once at start, then on the interval.
            let mut last_maintenance: Option<Instant> = None;

            while *running.lock().await {
                if last_maintenance.is_none_or(|at| at.elapsed() >= config.maintenance_interval) {
                    maintain(repo.as_ref(), &handlers, config.retention).await;
                    last_maintenance = Some(Instant::now());
                }

                // Hold a slot before claiming, so a claimed job is never left
                // waiting for one while its lease runs down.
                let Ok(permit) = slots.clone().acquire_owned().await else {
                    break;
                };

                let backoff = match repo.claim_job(&kinds, &lease).await {
                    Ok(Some(job)) => {
                        tokio::spawn(run_job(
                            repo.clone(),
                            handlers.clone(),
                            lease.clone(),
                            config.retention,
                            job,
                            permit,
                        ));
                        None
                    }
                    Ok(None) => {
                        drop(permit);
                        Some(config.poll_interval)
                    }
                    Err(error) => {
                        drop(permit);
                        tracing::error!("Failed to claim a job: {error:?}");
                        Some(ERROR_BACKOFF)
                    }
                };

                if let Some(backoff) = backoff {
                    tokio::time::sleep(backoff).await;
                }
            }
        });

        std::future::ready(Ok(spawned))
    }

    async fn stop(&mut self) -> Result<(), OperationOutcomeError> {
        *self.running.lock().await = false;
        Ok(())
    }
}
