use std::sync::Arc;

use etl::{
    config::{BatchConfig, PgConnectionConfig, PipelineConfig, TableSyncCopyConfig, TlsConfig},
    pipeline::Pipeline,
    store::both::memory::MemoryStore,
};
use haste_artifacts::search_parameters::MemoryResolver;
use haste_config::get_config;
use haste_fhir_search::elastic_search::ElasticSearchEngine;

use crate::es_search_destination::ESSearchDestination;
mod es_search_destination;

static PIPELINE_ID: u64 = 1;

pub enum ESSearchWorkerEnvironmentVariables {
    ElasticSearchURL,
    ElasticSearchUsername,
    ElasticSearchPassword,
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
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = get_config::<ESSearchWorkerEnvironmentVariables>("environment".into());
    let search_engine = ElasticSearchEngine::new(
        Arc::new(MemoryResolver::new()),
        Arc::new(haste_fhirpath::FPEngine::new()),
        &config
            .get(ESSearchWorkerEnvironmentVariables::ElasticSearchURL)
            .expect(&format!(
                "'{}' variable not set",
                String::from(ESSearchWorkerEnvironmentVariables::ElasticSearchURL)
            )),
        config
            .get(ESSearchWorkerEnvironmentVariables::ElasticSearchUsername)
            .expect(&format!(
                "'{}' variable not set",
                String::from(ESSearchWorkerEnvironmentVariables::ElasticSearchUsername)
            )),
        config
            .get(ESSearchWorkerEnvironmentVariables::ElasticSearchPassword)
            .expect(&format!(
                "'{}' variable not set",
                String::from(ESSearchWorkerEnvironmentVariables::ElasticSearchPassword)
            )),
    )
    .expect("Failed to create Elasticsearch client");

    let pg_config = PgConnectionConfig {
        host: "localhost".to_string(),
        port: 5432,
        name: "haste_health".to_string(),
        username: "postgres".to_string(),
        password: Some("postgres".to_string().into()), // Update this
        tls: TlsConfig {
            enabled: false,
            trusted_root_certs: String::new(),
        },
        keepalive: None,
    };

    let config = PipelineConfig {
        id: PIPELINE_ID,
        publication_name: "my_publication".to_string(),
        pg_connection: pg_config.clone(),
        batch: BatchConfig {
            max_size: 1000,
            max_fill_ms: 5000,
        },
        table_error_retry_delay_ms: 10000,
        table_error_retry_max_attempts: 5,
        max_table_sync_workers: 4,
        table_sync_copy: TableSyncCopyConfig::SkipAllTables,
    };

    let store = MemoryStore::new();
    // let store = PostgresStore::new(PIPELINE_ID, pg_config);
    let destination = ESSearchDestination::new(search_engine)
        .expect("Failed to create Elasticsearch destination");

    println!("Starting pipeline...");
    let mut pipeline = Pipeline::new(config, store, destination);
    pipeline.start().await?;
    pipeline.wait().await?;

    Ok(())
}
