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

/// Run background workers (search indexing, async jobs).
#[derive(Subcommand, Debug)]
pub(crate) enum WorkerCommands {
    /// Run the search-indexing and async-job workers. Default when no subcommand is given.
    Worker,
}

fn config_error(e: figment::Error) -> OperationOutcomeError {
    OperationOutcomeError::error(IssueType::exception(), e.to_string())
}

fn figment() -> Figment {
    Figment::new()
        .merge(Toml::file("haste.toml"))
        .merge(Env::prefixed("HASTE_"))
}

/// Runs the `worker` command group. `command` is `None` when the bare `worker`
/// invocation is used, which behaves the same as `worker worker`.
pub(crate) async fn run(command: &Option<WorkerCommands>) -> Result<(), OperationOutcomeError> {
    match command {
        None | Some(WorkerCommands::Worker) => {
            let indexing_config: Arc<search_indexing::WorkerEnvironment> =
                Arc::new(figment().extract().map_err(config_error)?);
            let indexing_worker = search_indexing::IndexingWorker::new(indexing_config).await?;

            // Jobs run through the server's own FHIR client, so the job
            // worker is built from the server's services (without the
            // index migrations the server itself owns).
            let server_config: Arc<ServerConfig> =
                Arc::new(figment().extract().map_err(config_error)?);
            let state = services::create_worker_services(server_config.clone()).await?;
            let handlers: Vec<Arc<dyn JobHandler>> =
                vec![Arc::new(OperationJobHandler::new(state.clone()))];
            let job_worker = AsyncJobWorker::new(
                state.repo.clone(),
                handlers,
                server_config.jobs.worker_config(),
            );

            let indexing = indexing_worker.run().await?;
            let jobs = job_worker.run().await?;

            // Either loop ending (they only end on a panic) ends the process,
            // so a supervisor restarts both rather than one running on alone.
            let failure = tokio::select! {
                result = indexing => result.err().map(|e| format!("Indexing worker task failed: {e:?}")),
                result = jobs => result.err().map(|e| format!("Async job worker task failed: {e:?}")),
            };

            match failure {
                Some(message) => Err(OperationOutcomeError::fatal(
                    IssueType::exception(),
                    message,
                )),
                None => Ok(()),
            }
        }
    }
}
