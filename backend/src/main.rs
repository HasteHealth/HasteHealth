#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::{collections::HashMap, time::Duration};

use clap::{Parser, Subcommand};
use haste_config::{Config, ConfigType, get_config};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use opentelemetry::KeyValue;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_otlp::{Protocol, WithHttpConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use reqwest::Url;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, registry::Registry};
use tracing_tree::HierarchicalLayer;

mod cli;
mod commands;

#[derive(Parser)]
#[command(version, about, long_about = None)] // Read from `Cargo.toml`
pub struct Cli {
    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Subcommand)]
enum CliCommand {
    /// Evaluate a FHIRPath expression against a FHIR resource read from stdin.
    FHIRPath {
        /// FHIRPath expression to evaluate
        fhirpath: String,
    },
    /// Code generators (Rust FHIR types, operations, TestScripts) used to build this crate.
    Generate {
        #[command(subcommand)]
        command: commands::codegen::CodeGen,
    },
    /// Run the FHIR server.
    Server {
        #[command(subcommand)]
        command: commands::server::ServerCommands,
    },
    /// Make FHIR REST API calls against the active profile's server.
    Api {
        #[command(subcommand)]
        command: commands::api::ApiCommands,
    },
    /// Manage named server connection profiles used by other commands.
    Config {
        #[command(subcommand)]
        command: commands::config::ConfigCommands,
    },
    /// Log in as a human user via the browser (authorization_code + PKCE flow).
    Login,
    /// Run background workers (search indexing, WAL processing).
    Worker {
        #[command(subcommand)]
        command: Option<commands::worker::WorkerCommands>,
    },
    /// Run FHIR TestScript resources against the active profile's server.
    Testscript {
        #[command(subcommand)]
        command: commands::testscript::TestScriptCommands,
    },
    /// Server-side administrative operations (tenants, users, clients, migrations).
    Admin {
        #[command(subcommand)]
        command: commands::admin::AdminCommands,
    },
    /// Bridge HL7v2 messages to and from the FHIR server.
    Hl7v2 {
        #[command(subcommand)]
        command: commands::hl7v2::HL7v2Commands,
    },
    /// Generate Markdown documentation for this CLI's commands.
    Doc {
        /// Output markdown file path
        #[arg(short, long)]
        output: String,
    },
}

enum CliEnvironmentVariables {
    SentryDSN,
    OTELEndpoint,
    OTELHeaders,
    LogType,
}

impl From<CliEnvironmentVariables> for String {
    fn from(value: CliEnvironmentVariables) -> Self {
        match value {
            CliEnvironmentVariables::SentryDSN => "SENTRY_DSN".to_string(),
            CliEnvironmentVariables::OTELEndpoint => "OTEL_ENDPOINT".to_string(),
            CliEnvironmentVariables::OTELHeaders => "OTEL_HEADERS".to_string(),
            CliEnvironmentVariables::LogType => "LOG_TYPE".to_string(),
        }
    }
}

struct OtelGuard {
    _tracer_provider: SdkTracerProvider,
    _logger_provider: SdkLoggerProvider,
}

fn otel_guard(config: &dyn Config<CliEnvironmentVariables>) -> Option<OtelGuard> {
    let endpoint = config.get(CliEnvironmentVariables::OTELEndpoint).ok()?;
    let headers_str = config
        .get(CliEnvironmentVariables::OTELHeaders)
        .unwrap_or_default();
    let headers = headers_str
        .split(',')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                Some((key.trim().to_string(), value.trim().to_string()))
            } else {
                None
            }
        })
        .collect::<HashMap<String, String>>();

    let root_otel_endpoint = Url::parse(&endpoint).expect("Invalid OTLP endpoint URL");

    // See https://opentelemetry.io/docs/specs/otlp/#otlphttp-request
    // v1/traces for spans, v1/logs for logs
    let mut trace_endpoint = root_otel_endpoint.clone();
    trace_endpoint
        .path_segments_mut()
        .expect("OTEL endpoint cannot be a base URL")
        .extend(&["v1", "traces"]);

    let mut log_endpoint = root_otel_endpoint.clone();
    log_endpoint
        .path_segments_mut()
        .expect("OTEL endpoint cannot be a base URL")
        .extend(&["v1", "logs"]);

    let oltp_span_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .with_endpoint(trace_endpoint.as_str())
        .with_headers(headers.clone())
        .with_timeout(Duration::from_secs(5))
        .build()
        .expect("Failed to create OpenTelemetry span exporter");

    let oltp_log_exporter = opentelemetry_otlp::LogExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .with_endpoint(log_endpoint.as_str())
        .with_headers(headers)
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

    Some(OtelGuard {
        _tracer_provider: tracer_provider,
        _logger_provider: logger_provider,
    })
}

fn inject_otel_subscriber<S>(
    subscriber: S,
    config: &dyn Config<CliEnvironmentVariables>,
) -> Option<OtelGuard>
where
    S: tracing::Subscriber + Send + Sync + 'static,
    for<'span> S: LookupSpan<'span>,
{
    if let Some(guard) = otel_guard(config) {
        let tracer = guard._tracer_provider.tracer("haste-health");
        let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
        let otel_logs = OpenTelemetryTracingBridge::new(&guard._logger_provider);

        tracing::subscriber::set_global_default(subscriber.with(telemetry).with(otel_logs))
            .unwrap();

        Some(guard)
    } else {
        tracing::subscriber::set_global_default(subscriber).unwrap();
        None
    }
}

fn setup_tracing(
    config: &dyn Config<CliEnvironmentVariables>,
) -> Result<Option<OtelGuard>, OperationOutcomeError> {
    let log_type = config
        .get(CliEnvironmentVariables::LogType)
        .unwrap_or("JSON".to_string());

    let subscriber = Registry::default()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")));

    match log_type.as_str() {
        "TREE" => Ok(inject_otel_subscriber(
            subscriber.with(HierarchicalLayer::new(2)),
            config,
        )),
        "JSON" => Ok(inject_otel_subscriber(
            subscriber.with(tracing_subscriber::fmt::Layer::default().json()),
            config,
        )),
        _ => Err(OperationOutcomeError::fatal(
            IssueType::invalid(),
            "Invalid log type specified in environment variable LOG_TYPE. Supported values are 'TREE' and 'JSON'.".to_string(),
        )),
    }
}

fn main() -> Result<(), OperationOutcomeError> {
    let config = get_config(ConfigType::Environment);

    let cli = Cli::parse();
    let cli_state = cli::state::CLI_STATE.clone();

    let sentry_location = config.get(CliEnvironmentVariables::SentryDSN);

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
            let _otel_provider = setup_tracing(config.as_ref())?;
            commands::run(cli_state, &cli.command).await
        })
}
