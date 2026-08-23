pub(crate) mod admin;
pub(crate) mod api;
pub(crate) mod codegen;
pub(crate) mod config;
pub(crate) mod doc;
pub(crate) mod fhirpath;
pub(crate) mod hl7v2;
pub(crate) mod login;
pub(crate) mod server;
pub(crate) mod testscript;
pub(crate) mod worker;

use crate::{CliCommand, cli::state::CliState};
use haste_fhir_operation_error::OperationOutcomeError;
use std::sync::Arc;
use tokio::sync::Mutex;

pub(crate) async fn run(
    state: Arc<Mutex<CliState>>,
    command: &CliCommand,
) -> Result<(), OperationOutcomeError> {
    match command {
        CliCommand::Doc { output } => doc::run(output).await,
        CliCommand::FHIRPath { fhirpath } => fhirpath::run(fhirpath).await,
        CliCommand::Generate { command } => codegen::run(command).await,
        CliCommand::Server { command } => server::run(command).await,
        CliCommand::Worker { command } => worker::run(command).await,
        CliCommand::Config { command } => config::run(state, command).await,
        CliCommand::Login => login::run(state).await,
        CliCommand::Api { command } => api::run(state, command).await,
        CliCommand::Testscript { command } => testscript::run(state, command).await,
        CliCommand::Admin { command } => admin::run(command).await,
        CliCommand::Hl7v2 { command } => hl7v2::run(state, command).await,
    }
}
