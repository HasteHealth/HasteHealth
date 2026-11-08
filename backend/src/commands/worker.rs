use std::{sync::Arc, time::Duration};

use clap::Subcommand;
use figment::{
    Figment,
    providers::{Env, Format as _, Toml},
};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_repository::pg::PGConnection;
use haste_server::{
    config::ServerConfig,
    document_storage::{self, BinaryStorage, DocumentStorageConfig},
    jobs::operation::OperationJobHandler,
    services,
};
use haste_worker::{
    async_jobs::{AsyncJobWorker, JobHandler},
    search_indexing,
    traits::Worker as _,
};
use tokio::task::{JoinError, JoinHandle};

/// Run background workers. With no subcommand, runs all of them in one process.
#[derive(Subcommand, Debug)]
pub(crate) enum WorkerCommands {
    /// Run only the search-indexing worker.
    Indexing,
    /// Run only the async-job worker.
    AsyncJobs,
}

fn config_error(e: figment::Error) -> OperationOutcomeError {
    OperationOutcomeError::error(IssueType::exception(), e.to_string())
}

fn figment() -> Figment {
    Figment::new()
        .merge(Toml::file("haste.toml"))
        .merge(Env::prefixed("HASTE_"))
}

async fn start_indexing() -> Result<JoinHandle<()>, OperationOutcomeError> {
    let config: Arc<search_indexing::WorkerEnvironment> =
        Arc::new(figment().extract().map_err(config_error)?);

    search_indexing::IndexingWorker::new(config)
        .await?
        .run()
        .await
}

async fn start_async_jobs() -> Result<JoinHandle<()>, OperationOutcomeError> {
    // Jobs run through the server's own FHIR client, so the job worker is
    // built from the server's services (without the index migrations the
    // server itself owns).
    let config: Arc<ServerConfig> = Arc::new(figment().extract().map_err(config_error)?);
    let state = services::create_worker_services(config.clone()).await?;
    spawn_document_sweep(
        state.repo.clone(),
        state.binary_storage.clone(),
        &config.document_storage,
    );
    let handlers: Vec<Arc<dyn JobHandler>> =
        vec![Arc::new(OperationJobHandler::new(state.clone()))];

    AsyncJobWorker::new(state.repo.clone(), handlers, config.jobs.worker_config())
        .run()
        .await
}

/// Removes orphaned Binary documents every `sweep_interval_seconds`, first
/// one interval after the worker starts. Each async-job worker runs its own;
/// a sweep only deletes documents no resource version refers to, so two at
/// once do no harm. A failed sweep is logged and tried again next interval.
fn spawn_document_sweep(
    repo: Arc<PGConnection>,
    storage: Option<BinaryStorage>,
    config: &DocumentStorageConfig,
) {
    let Some(storage) = storage else {
        return;
    };
    if config.sweep_interval_seconds == 0 {
        return;
    }
    let interval = Duration::from_secs(config.sweep_interval_seconds);
    let grace = Duration::from_secs(config.sweep_grace_seconds);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(interval).await;
            match document_storage::sweep(&repo, &storage, grace, false).await {
                Ok(report) => tracing::info!(?report, "document storage sweep finished"),
                Err(error) => tracing::error!(%error, "document storage sweep failed"),
            }
        }
    });
}

/// A worker's loop only ends on a panic, which fails the command.
fn finished(name: &str, result: Result<(), JoinError>) -> Result<(), OperationOutcomeError> {
    result.map_err(|e| {
        OperationOutcomeError::fatal(
            IssueType::exception(),
            format!("{name} worker task failed: {e:?}"),
        )
    })
}

/// Runs the `worker` command group. `command` is `None` when the bare `worker`
/// invocation is used, which runs every worker.
pub(crate) async fn run(command: &Option<WorkerCommands>) -> Result<(), OperationOutcomeError> {
    match command {
        None => {
            let indexing = start_indexing().await?;
            let jobs = start_async_jobs().await?;

            // Either loop ending ends the process, so a supervisor restarts
            // both rather than one running on alone.
            tokio::select! {
                result = indexing => finished("Indexing", result),
                result = jobs => finished("Async job", result),
            }
        }
        Some(WorkerCommands::Indexing) => finished("Indexing", start_indexing().await?.await),
        Some(WorkerCommands::AsyncJobs) => finished("Async job", start_async_jobs().await?.await),
    }
}
