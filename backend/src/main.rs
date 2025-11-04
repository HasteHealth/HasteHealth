use clap::{Parser, Subcommand};
use oxidized_fhir_operation_error::OperationOutcomeError;

mod codegen_commands;
mod fhirpath_commands;

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
        command: codegen_commands::CodeGen,
    },
}

#[tokio::main]
async fn main() -> Result<(), OperationOutcomeError> {
    let cli = Cli::parse();
    match &cli.command {
        CLICommand::FHIRPath { fhirpath } => fhirpath_commands::fhirpath(fhirpath),
        CLICommand::Generate { command } => codegen_commands::codegen(command).await,
    }
}
