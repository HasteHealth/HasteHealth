use std::time::Duration;

use etl::{
    config::{BatchConfig, PgConnectionConfig, PipelineConfig, TableSyncCopyConfig, TlsConfig},
    destination::memory::MemoryDestination,
    pipeline::Pipeline,
    store::both::{memory::MemoryStore, postgres::PostgresStore},
};
mod es_search_destination;

static PIPELINE_ID: u64 = 1;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
        pg_connection: pg_config,
        batch: BatchConfig {
            max_size: 1000,
            max_fill_ms: 5000,
        },
        table_error_retry_delay_ms: 10000,
        table_error_retry_max_attempts: 5,
        max_table_sync_workers: 4,
        table_sync_copy: TableSyncCopyConfig::SkipAllTables,
    };

    let store = PostgresStore::new(PIPELINE_ID, pg_config);
    let destination = MemoryDestination::new();

    // Print destination contents periodically
    let dest_clone = destination.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            let rows = dest_clone.table_rows().await;
            let events = dest_clone.events().await;
            println!("\n--- Destination State ---");
            println!("Tables: {}, Events: {}", rows.len(), events.len());

            let event_types = events.iter().map(|e| e.event_type()).collect::<Vec<_>>();

            println!("rows : {:?} events: {:?}", rows, event_types);

            for (table_id, table_rows) in &rows {
                println!("  Table {}: {} rows", table_id.0, table_rows.len());
            }
        }
    });

    println!("Starting pipeline...");
    let mut pipeline = Pipeline::new(config, store, destination);
    pipeline.start().await?;
    pipeline.wait().await?;

    Ok(())
}
