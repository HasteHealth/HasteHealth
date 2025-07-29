use std::{sync::Arc, time::Instant};

use crate::lock::{LockKind, LockProvider, postgres::PostgresLockProvider};
use oxidized_config::get_config;
use oxidized_fhir_model::r4::{
    sqlx::FHIRJson,
    types::{Identifier, Resource},
};
use oxidized_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use oxidized_reflect::MetaValue;
use rayon::prelude::*;
use sqlx::{Connection, query_as, types::time::OffsetDateTime};
// use tokio::time::sleep;
// use std::time::Duration;

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

struct TenantReturn {
    id: String,
    created_at: OffsetDateTime,
}

async fn get_tenants(
    client: &mut sqlx::PgConnection,
    cursor: &OffsetDateTime,
    count: usize,
) -> Result<Vec<TenantReturn>, OperationOutcomeError> {
    let result = query_as!(
        TenantReturn,
        r#"SELECT id, created_at FROM tenants WHERE created_at > $1 ORDER BY created_at DESC LIMIT $2"#,
        cursor,
        count as i64
    )
    .fetch_all(client)
    .await
    .map_err(IndexingWorkerError::from)?;

    Ok(result)
}

#[tokio::main]
pub async fn main() {
    // Initialize the PostgreSQL connection pool
    let config = get_config("environment".into());
    let mut pg_connection = sqlx::PgConnection::connect(&config.get("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to the database");
    let mut cursor = OffsetDateTime::UNIX_EPOCH;
    let tenants_limit: usize = 100;

    let fp_engine = Arc::new(oxidized_fhirpath::FPEngine::new());

    loop {
        let tenants_to_check = get_tenants(&mut pg_connection, &cursor, tenants_limit)
            .await
            .expect("Failed to get tenants.");

        if tenants_to_check.is_empty() || tenants_to_check.len() < tenants_limit {
            cursor = OffsetDateTime::UNIX_EPOCH; // Reset cursor if no tenants found
        } else {
            cursor = tenants_to_check[0].created_at;
        }

        for tenant in tenants_to_check {
            let fp_engine = fp_engine.clone();
            pg_connection
                .transaction(|t| {
                    Box::pin(async move {
                        let mut provider = PostgresLockProvider::new(t);
                        let locks = provider
                            .get_available(LockKind::System, vec![tenant.id.as_str().into()])
                            .await?;

                        if locks.is_empty() {
                            return Ok(());
                        }

                        let resources = get_resource_sequence(t, "tenant", 0, Some(100)).await?;

                        println!("Available locks: {:?}", locks);
                        println!("Retrieved resources: {:?}", resources.len());

                        // sleep(Duration::from_millis(1000)).await;
                        let start = Instant::now();

                        let index_set = resources
                            .par_iter()
                            .flat_map(|r| {
                                let context: Vec<&dyn MetaValue> = vec![r];
                                let result = fp_engine.evaluate(
                                    "$this.identifier.where($this.value = '123')",
                                    context,
                                );

                                if let Ok(values) = result {
                                    let ids = values
                                        .iter()
                                        .filter_map(|v| v.as_any().downcast_ref::<Identifier>())
                                        .map(|id| id.clone())
                                        .collect::<Vec<_>>();
                                    ids
                                } else {
                                    vec![]
                                }
                            })
                            .collect::<Vec<_>>();

                        println!("Evaluation took: {:?}", start.elapsed());
                        let ret: Result<(), IndexingWorkerError> = Ok(());
                        ret
                    })
                })
                .await
                .expect("Transaction failed");
        }
    }
}
