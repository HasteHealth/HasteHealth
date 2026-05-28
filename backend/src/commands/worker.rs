use clap::Subcommand;
// use haste_config::get_config;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_worker::{search_indexing, traits::Worker as _};
// use haste_wal_worker::{WALWorkerEnvironmentVariables, wal_worker};

#[derive(Subcommand, Debug)]
pub enum WorkerCommands {
    Worker,
    WalWorker,
}

pub async fn worker(command: &Option<WorkerCommands>) -> Result<(), OperationOutcomeError> {
    match command {
        None | Some(WorkerCommands::Worker) => indexing_worker()
            .await
            .expect("Failed to run indexing worker"),
        Some(WorkerCommands::WalWorker) => todo!(),
    };

    Ok(())
}

async fn indexing_worker() -> Result<(), OperationOutcomeError> {
    let indexing_worker = search_indexing::IndexingWorker::new().await?;

    indexing_worker.run().await?;

    Ok(())
}

// async fn create_wal_worker() -> Result<(), Box<dyn std::error::Error>> {
//     let config = get_config::<WALWorkerEnvironmentVariables>("environment".into());

//     let connection_url = config
//         .get(WALWorkerEnvironmentVariables::DatabaseURL)
//         .expect(&format!(
//             "'{}' variable not set",
//             String::from(WALWorkerEnvironmentVariables::DatabaseURL)
//         ));

//     let slot_name = config
//         .get(WALWorkerEnvironmentVariables::PGSlotName)
//         .expect(&format!(
//             "'{}' variable not set",
//             String::from(WALWorkerEnvironmentVariables::PGSlotName)
//         ));
//     let publication_name = config
//         .get(WALWorkerEnvironmentVariables::PGPublicationName)
//         .expect(&format!(
//             "'{}' variable not set",
//             String::from(WALWorkerEnvironmentVariables::PGPublicationName)
//         ));

//     wal_worker(slot_name, publication_name, &connection_url).await;

//     Ok(())
// }
