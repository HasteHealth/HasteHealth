use std::{
    path::PathBuf,
    sync::{Arc, LazyLock},
};

use clap::{Parser, Subcommand};
use haste_config::{ConfigType, get_config};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_server::auth_n::oidc::routes::discovery::WellKnownDiscoveryDocument;
use tokio::sync::Mutex;

use crate::commands::config::{CLIConfiguration, load_config};

mod client;
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
    Worker {
        #[command(subcommand)]
        command: Option<commands::worker::WorkerCommands>,
    },
    Testscript {
        #[command(subcommand)]
        command: commands::testscript::TestScriptCommands,
    },
    Admin {
        #[command(subcommand)]
        command: commands::admin::AdminCommands,
    },
}

static CONFIG_LOCATION: LazyLock<PathBuf> = LazyLock::new(|| {
    let config_dir = std::env::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".haste_health");

    std::fs::create_dir_all(&config_dir).expect("Failed to create config directory");

    config_dir.join("config.toml")
});

pub struct CLIState {
    config: CLIConfiguration,
    access_token: Option<String>,
    well_known_document: Option<WellKnownDiscoveryDocument>,
}

impl CLIState {
    pub fn new(config: CLIConfiguration) -> Self {
        CLIState {
            config,
            access_token: None,
            well_known_document: None,
        }
    }
}

static CLI_STATE: LazyLock<Arc<Mutex<CLIState>>> = LazyLock::new(|| {
    let config = load_config(&CONFIG_LOCATION);

    Arc::new(Mutex::new(CLIState::new(config)))
});

enum CLIEnvironmentVariables {
    SentryDSN,
}

impl From<CLIEnvironmentVariables> for String {
    fn from(value: CLIEnvironmentVariables) -> Self {
        match value {
            CLIEnvironmentVariables::SentryDSN => "SENTRY_DSN".to_string(),
        }
    }
}

fn main() -> Result<(), OperationOutcomeError> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    let config = CLI_STATE.clone();
    let env = get_config(ConfigType::Environment);
    let sentry_location = env.get(CLIEnvironmentVariables::SentryDSN);

    // let subscriber = tracing_subscriber::FmtSubscriber::new();
    // tracing::subscriber::set_global_default(subscriber).unwrap();

    let _guard = sentry::init((
        sentry_location.unwrap_or_default(),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            // Capture user IPs and potentially sensitive headers when using HTTP server integrations
            // see https://docs.sentry.io/platforms/rust/data-management/data-collected for more info
            send_default_pii: true,
            ..Default::default()
        },
    ));

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        // 8MB stack size
        .thread_stack_size(1024 * 8000)
        .build()
        .unwrap()
        .block_on(async {
            match &cli.command {
                CLICommand::FHIRPath { fhirpath } => commands::fhirpath::fhirpath(fhirpath).await,
                CLICommand::Generate { command } => commands::codegen::codegen(command).await,
                CLICommand::Server { command } => commands::server::server(command).await,
                CLICommand::Worker { command } => commands::worker::worker(command).await,
                CLICommand::Config { command } => commands::config::config(&config, command).await,
                CLICommand::Api { command } => commands::api::api_commands(config, command).await,
                CLICommand::Testscript { command } => {
                    commands::testscript::testscript_commands(config, command).await
                }
                CLICommand::Admin { command } => commands::admin::admin(command).await,
            }
        })
}
