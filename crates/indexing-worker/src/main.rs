use std::time::Duration;

use crate::lock::{LockKind, LockProvider, postgres::PostgresLockProvider};
use oxidized_config::get_config;
use oxidized_fhir_model::r4::{sqlx::FHIRJson, types::Resource};
use oxidized_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use sqlx::{Connection, query_as};
use tokio::time::sleep;

mod lock;

#[derive(OperationOutcomeError, Debug)]
pub enum IndexingWorkerError {
    #[fatal(code = "exception", diagnostic = "Database error: {arg0}")]
    DatabaseConnectionError(#[from] sqlx::Error),
    #[fatal(code = "exception", diagnostic = "Lock error: {arg0}")]
    OperationError(#[from] OperationOutcomeError),
}

struct ReturnV {
    resource: FHIRJson<Resource>,
}

/// Retrieves a sequence of resources from the database.
/// Must have sequence value greater than `cur_sequence`.
async fn get_resource_sequence(
    client: &mut sqlx::PgConnection,
    tenant_id: &str,
    cur_sequence: i64,
    count: Option<u64>,
) -> Result<Vec<Resource>, OperationOutcomeError> {
    let result = query_as!(
        ReturnV,
        r#"SELECT resource as "resource: FHIRJson<Resource>" FROM resources WHERE tenant = $1 AND sequence > $2 LIMIT $3"#,
        tenant_id,
        cur_sequence,
        count.unwrap_or(100) as i64
    )
    .fetch_all(client)
    .await
    .map_err(IndexingWorkerError::from)?;

    Ok(result
        .into_iter()
        .map(|r| r.resource.0)
        .into_iter()
        .collect::<Vec<_>>()
        .into())
}

#[tokio::main]
pub async fn main() {
    // Initialize the PostgreSQL connection pool
    let config = get_config("environment".into());
    let mut pg_connection = sqlx::PgConnection::connect(&config.get("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to the database");

    loop {
        pg_connection
            .transaction(|t| {
                Box::pin(async move {
                    let mut provider = PostgresLockProvider::new(t);
                    let locks = provider
                        .get_available(LockKind::System, vec!["tenant".into()])
                        .await?;

                    if locks.is_empty() {
                        return Ok(());
                    }

                    let resources = get_resource_sequence(t, "tenant", 0, Some(100)).await?;

                    println!("Available locks: {:?}", locks);
                    println!("Retrieved resources: {:?}", resources.len());

                    // sleep(Duration::from_millis(1000)).await;

                    let ret: Result<(), IndexingWorkerError> = Ok(());

                    ret
                })
            })
            .await
            .expect("Transaction failed");
    }
}
