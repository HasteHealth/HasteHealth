use std::sync::Arc;

use clap::Subcommand;
use figment::{
    Figment,
    providers::{Env, Format as _, Toml},
};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_worker::{search_indexing, traits::Worker as _};

/// Run background workers (search indexing).
#[derive(Subcommand, Debug)]
pub(crate) enum WorkerCommands {
    /// Run the search-indexing worker. Default when no subcommand is given.
    Worker,
}

/// Runs the `worker` command group. `command` is `None` when the bare `worker`
/// invocation is used, which behaves the same as `worker worker`.
pub(crate) async fn run(command: &Option<WorkerCommands>) -> Result<(), OperationOutcomeError> {
    match command {
        None | Some(WorkerCommands::Worker) => {
            let config: Arc<search_indexing::WorkerEnvironment> = Arc::new(
                Figment::new()
                    .merge(Toml::file("haste.toml"))
                    .merge(Env::prefixed("HASTE_"))
                    .extract()
                    .map_err(|e| {
                        OperationOutcomeError::error(IssueType::exception(), e.to_string())
                    })?,
            );

            let indexing_worker = search_indexing::IndexingWorker::new(config).await?;

            let handler = indexing_worker.run().await?;

            handler.await.map_err(|e| {
                OperationOutcomeError::fatal(
                    haste_fhir_model::r4::generated::terminology::IssueType::exception(),
                    format!("Worker task failed: {:?}", e),
                )
            })?;

            Ok(())
        }
    }
}
