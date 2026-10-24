use std::sync::Arc;

use clap::Subcommand;
use figment::{
    Figment,
    providers::{Env, Format as _, Toml},
};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_server::{config::ServerConfig, jobs::operation::OperationJobHandler, services};
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
    let handlers: Vec<Arc<dyn JobHandler>> =
        vec![Arc::new(OperationJobHandler::new(state.clone()))];

    AsyncJobWorker::new(state.repo.clone(), handlers, config.jobs.worker_config())
        .run()
        .await
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
