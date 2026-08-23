use std::sync::Arc;

use clap::Subcommand;
use figment::{
    Figment,
    providers::{Env, Format as _, Toml},
};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_server::{config::ServerConfig, server};

/// Run the FHIR server.
#[derive(Subcommand, Debug)]
pub(crate) enum ServerCommands {
    /// Start the HTTP server. Configuration is read from `haste.toml` and `HASTE_*` env vars.
    Start {
        /// Port to listen on. Defaults to 3000.
        #[arg(short, long)]
        port: Option<u16>,
    },
}

/// Runs the `server` command group.
pub(crate) async fn run(command: &ServerCommands) -> Result<(), OperationOutcomeError> {
    let config: ServerConfig = Figment::new()
        .merge(Toml::file("haste.toml"))
        .merge(Env::prefixed("HASTE_"))
        .extract()
        .map_err(|e| OperationOutcomeError::error(IssueType::exception(), e.to_string()))?;

    match &command {
        ServerCommands::Start { port } => {
            server::serve(Arc::new(config), port.unwrap_or(3000)).await
        }
    }
}
