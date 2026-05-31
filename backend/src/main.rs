use std::{
    path::PathBuf,
    sync::{Arc, LazyLock},
    time::Duration,
};

use clap::{Parser, Subcommand};
use haste_config::{ConfigType, get_config};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_server::auth_n::oidc::routes::discovery::WellKnownDiscoveryDocument;
use opentelemetry::KeyValue;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::Protocol;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tokio::sync::Mutex;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, registry::Registry};
use tracing_tree::HierarchicalLayer;

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

struct OtelGuard {
    _tracer_provider: SdkTracerProvider,
    _logger_provider: SdkLoggerProvider,
}

fn otel_subscriber() -> OtelGuard {
    let oltp_span_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .with_timeout(Duration::from_secs(5))
        .build()
        .expect("Failed to create OpenTelemetry span exporter");

    let oltp_log_exporter = opentelemetry_otlp::LogExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .with_timeout(Duration::from_secs(5))
        .build()
        .expect("Failed to create OpenTelemetry log exporter");

    let resource = Resource::builder()
        .with_attribute(KeyValue::new("service.name", "haste-health"))
        .build();

    let tracer_provider = SdkTracerProvider::builder()
        .with_resource(resource.clone())
        .with_batch_exporter(oltp_span_exporter)
        .build();

    let logger_provider = SdkLoggerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(oltp_log_exporter)
        .build();

    opentelemetry::global::set_tracer_provider(tracer_provider.clone());

    let tracer = tracer_provider.tracer("haste-health");

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let otel_logs = OpenTelemetryTracingBridge::new(&logger_provider);

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let subscriber = Registry::default()
        .with(env_filter)
        .with(tracing_subscriber::fmt::Layer::default())
        .with(telemetry)
        .with(otel_logs);

    tracing::subscriber::set_global_default(subscriber).unwrap();

    OtelGuard {
        _tracer_provider: tracer_provider,
        _logger_provider: logger_provider,
    }
}

#[allow(dead_code)]
fn tree_subscriber() -> impl tracing::Subscriber {
    let subscriber = Registry::default()
        .with(HierarchicalLayer::new(2))
        .with(EnvFilter::from_default_env());
    subscriber
}

fn main() -> Result<(), OperationOutcomeError> {
    let env = get_config(ConfigType::Environment);

    let cli = Cli::parse();
    let config = CLI_STATE.clone();

    let sentry_location = env.get(CLIEnvironmentVariables::SentryDSN);

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
            // Set up tracing with a tree subscriber in debug builds for better visibility during development,
            // and OpenTelemetry in release builds for production monitoring.
            // #[cfg(debug_assertions)]
            // tracing::subscriber::set_global_default(tree_subscriber()).unwrap();
            // #[cfg(not(debug_assertions))]
            let _otel_provider = otel_subscriber();
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
