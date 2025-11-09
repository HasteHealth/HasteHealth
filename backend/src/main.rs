use std::{
    path::PathBuf,
    sync::{Arc, LazyLock, Mutex},
};

use clap::{Parser, Subcommand};
use oxidized_fhir_operation_error::OperationOutcomeError;

use crate::commands::config::{CLIConfiguration, load_config};

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
    Api {
        #[command(subcommand)]
        command: commands::api::ApiCommands,
    },
    Config {
        #[command(subcommand)]
        command: commands::config::ConfigCommands,
    },
    Worker {},
}

static CONFIG_DIRECTORY: LazyLock<PathBuf> = LazyLock::new(|| {
    let config_dir = std::env::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".oxidized_health")
        .join("config");
    std::fs::create_dir_all(&config_dir).expect("Failed to create config directory");
    config_dir
});

static CLI_CONFIG: LazyLock<Arc<Mutex<CLIConfiguration>>> = LazyLock::new(|| {
    let config_path = CONFIG_DIRECTORY.join("config.toml");
    let config = load_config(&config_path).unwrap_or_default();
    Arc::new(Mutex::new(config))
});

#[tokio::main]
async fn main() -> Result<(), OperationOutcomeError> {
    let cli = Cli::parse();
    let config = CLI_CONFIG.clone();

    println!("Using config {:#?}", config);
    match &cli.command {
        CLICommand::FHIRPath { fhirpath } => commands::fhirpath::fhirpath(fhirpath),
        CLICommand::Generate { command } => commands::codegen::codegen(command).await,
        CLICommand::Server { command } => commands::server::server(command).await,
        CLICommand::Worker {} => commands::worker::worker().await,
        CLICommand::Config { command } => todo!(),
        CLICommand::Api { command } => todo!(),
    }
}
