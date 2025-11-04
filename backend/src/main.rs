use clap::{Parser, Subcommand};
use oxidized_fhir_operation_error::OperationOutcomeError;

mod commands;

#[derive(Parser)]
#[command(version, about, long_about = None)] // Read from `Cargo.toml`
struct Cli {
    #[command(subcommand)]
    command: CLICommand,
}

#[derive(Subcommand)]
enum CLICommand {
    /// Data gets pulled from stdin.
    FHIRPath {
        /// lists test values
        fhirpath: String,
    },
    Generate {
        /// Input FHIR StructureDefinition file (JSON)
        #[command(subcommand)]
        command: commands::codegen::CodeGen,
    },
    Server {
        #[command(subcommand)]
        command: commands::server::ServerCommands,
    },
    Worker {},
}

#[tokio::main]
async fn main() -> Result<(), OperationOutcomeError> {
    let cli = Cli::parse();
    match &cli.command {
        CLICommand::FHIRPath { fhirpath } => commands::fhirpath::fhirpath(fhirpath),
        CLICommand::Generate { command } => commands::codegen::codegen(command).await,
        CLICommand::Server { command } => commands::server::server(command).await,
        CLICommand::Worker {} => commands::worker::worker().await,
    }
}
