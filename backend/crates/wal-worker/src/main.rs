use std::sync::Arc;

use crate::es_search_destination::ESSearchDestination;
use etl::{
    config::{
        BatchConfig, InvalidatedSlotBehavior, MemoryBackpressureConfig, PgConnectionConfig,
        PipelineConfig, ReplicationSlotConfig, TableSyncCopyConfig, TcpKeepaliveConfig, TlsConfig,
    },
    pipeline::Pipeline,
    store::MemoryStore,
};
use haste_config::{Config, get_config};
use haste_fhir_search::elastic_search::{
    ElasticSearchEngine, create_es_client,
    search_parameter_resolver::ElasticSearchParameterResolver,
};
use haste_repository::pg::PGConnection;
use sqlx::{Pool, Postgres};
use sqlx_postgres::PgPoolOptions;
use tokio::sync::OnceCell;
use tracing::info;

mod es_search_destination;

static PIPELINE_ID: u64 = 1;

enum ESSearchWorkerEnvironmentVariables {
    ElasticSearchURL,
    ElasticSearchUsername,
    ElasticSearchPassword,
    // Database url
    DataBaseURL,
}

impl From<ESSearchWorkerEnvironmentVariables> for String {
    fn from(value: ESSearchWorkerEnvironmentVariables) -> Self {
        match value {
            ESSearchWorkerEnvironmentVariables::ElasticSearchURL => "ELASTICSEARCH_URL".to_string(),
            ESSearchWorkerEnvironmentVariables::ElasticSearchUsername => {
                "ELASTICSEARCH_USERNAME".to_string()
            }
            ESSearchWorkerEnvironmentVariables::ElasticSearchPassword => {
                "ELASTICSEARCH_PASSWORD".to_string()
            }
            ESSearchWorkerEnvironmentVariables::DataBaseURL => "DATABASE_URL".to_string(),
        }
    }
}

static POOL: OnceCell<Pool<Postgres>> = OnceCell::const_new();

async fn get_pool(
    config: &dyn Config<ESSearchWorkerEnvironmentVariables>,
) -> &'static Pool<Postgres> {
    POOL.get_or_init(async || {
        let database_url = config
            .get(ESSearchWorkerEnvironmentVariables::DataBaseURL)
            .unwrap_or_else(|_| {
                panic!(
                    "'{}' must be set",
                    String::from(ESSearchWorkerEnvironmentVariables::DataBaseURL)
                )
            });

        info!("Connecting to postgres database");
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to create database connection pool")
    })
    .await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = get_config::<ESSearchWorkerEnvironmentVariables>("environment".into());
    let pool = Arc::new(PGConnection::pool(get_pool(config.as_ref()).await.clone()));
    let es_client = create_es_client(
        &config
            .get(ESSearchWorkerEnvironmentVariables::ElasticSearchURL)
            .unwrap_or_else(|_| {
                panic!(
                    "'{}' variable not set",
                    String::from(ESSearchWorkerEnvironmentVariables::ElasticSearchURL)
                )
            }),
        config
            .get(ESSearchWorkerEnvironmentVariables::ElasticSearchUsername)
            .unwrap_or_else(|_| {
                panic!(
                    "'{}' variable not set",
                    String::from(ESSearchWorkerEnvironmentVariables::ElasticSearchUsername)
                )
            }),
        config
            .get(ESSearchWorkerEnvironmentVariables::ElasticSearchPassword)
            .unwrap_or_else(|_| {
                panic!(
                    "'{}' variable not set",
                    String::from(ESSearchWorkerEnvironmentVariables::ElasticSearchPassword)
                )
            }),
    )
    .expect("Failed to create Elasticsearch client");

    let pg_config = PgConnectionConfig {
        host: "localhost".to_string(),
        hostaddr: None,
        port: 5432,
        name: "haste_health".to_string(),
        username: "postgres".to_string(),
        password: Some("postgres".to_string().into()), // Update this
        tls: TlsConfig {
            enabled: false,
            trusted_root_certs: String::new(),
        },
        keepalive: TcpKeepaliveConfig::default(),
    };

    let search_engine = ElasticSearchEngine::new(
        Arc::new(ElasticSearchParameterResolver::new(es_client.clone(), pool)),
        Arc::new(haste_fhirpath::FPEngine::new()),
        es_client,
        // This worker only indexes documents; it never calls `migrate`.
        false,
    );

    let config = PipelineConfig {
        id: PIPELINE_ID,
        publication_name: "my_publication".to_string(),
        pg_connection: pg_config.clone(),
        store_pg_connection: None,
        replication_slot: ReplicationSlotConfig::default(),
        batch: BatchConfig {
            max_fill_ms: 5000,
            ..BatchConfig::default()
        },
        table_error_retry_delay_ms: 10000,
        table_error_retry_max_attempts: 5,
        max_table_sync_workers: 4,
        max_copy_connections_per_table: PipelineConfig::DEFAULT_MAX_COPY_CONNECTIONS_PER_TABLE,
        memory_refresh_interval_ms: PipelineConfig::DEFAULT_MEMORY_REFRESH_INTERVAL_MS,
        table_sync_monitor_refresh_interval_ms:
            PipelineConfig::DEFAULT_TABLE_SYNC_MONITOR_REFRESH_INTERVAL_MS,
        replication_lag_refresh_interval_ms:
            PipelineConfig::DEFAULT_REPLICATION_LAG_REFRESH_INTERVAL_MS,
        settle_durable_interval_ms: None,
        memory_backpressure: Some(MemoryBackpressureConfig::default()),
        table_sync_copy: TableSyncCopyConfig::SkipAllTables,
        invalidated_slot_behavior: InvalidatedSlotBehavior::default(),
        // Installs the etl schema and DDL trigger that every pipeline needs.
        run_source_migrations: true,
        drain_on_shutdown: PipelineConfig::DEFAULT_DRAIN_ON_SHUTDOWN,
    };

    let store = MemoryStore::new();
    // let store = PostgresStore::new(PIPELINE_ID, pg_config);
    let destination = ESSearchDestination::new(search_engine);

    println!("Starting pipeline...");
    let mut pipeline = Pipeline::new(config, store, destination);
    pipeline.start().await?;
    pipeline.wait().await?;

    Ok(())
}
