use crate::{
    conversion::InsertableIndex,
    indexing_lock::{IndexLockProvider, postgres::PostgresIndexLockProvider},
};
use elasticsearch::{
    BulkOperation, BulkParts, Elasticsearch,
    auth::Credentials,
    cert::CertificateValidation,
    http::{
        Url,
        transport::{SingleNodeConnectionPool, TransportBuilder},
    },
    indices::IndicesCreateParts,
};
use oxidized_config::get_config;
use oxidized_fhir_model::r4::{sqlx::FHIRJson, types::Resource};
use oxidized_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use oxidized_fhir_search_parameters::R4_SEARCH_PARAMETERS;
use oxidized_fhirpath::FHIRPathError;
use rayon::prelude::*;
use sqlx::{Connection, query_as, types::time::OffsetDateTime};
use std::{collections::HashMap, sync::Arc, time::Instant};

mod conversion;
mod indexing_lock;

#[derive(OperationOutcomeError, Debug)]
pub enum IndexingWorkerError {
    #[fatal(code = "exception", diagnostic = "Database error: {arg0}")]
    DatabaseConnectionError(#[from] sqlx::Error),
    #[fatal(code = "exception", diagnostic = "Lock error: {arg0}")]
    OperationError(#[from] OperationOutcomeError),
    #[fatal(code = "exception", diagnostic = "Elasticsearch error: {arg0}")]
    ElasticsearchError(#[from] elasticsearch::Error),
    #[fatal(code = "exception", diagnostic = "FHIRPath error: {arg0}")]
    FHIRPathError(#[from] FHIRPathError),
}

#[derive(sqlx::Type)]
#[sqlx(type_name = "fhir_method", rename_all = "lowercase")]
enum FHIRMethod {
    Create,
    Read,
    Update,
    Delete,
}

struct ReturnV {
    id: String,
    resource: FHIRJson<Resource>,
    sequence: i64,
    fhir_method: FHIRMethod,
}

static R4_FHIR_INDEX: &str = "r4_search_index";

/// Retrieves a sequence of resources from the database.
/// Must have sequence value greater than `cur_sequence`.
async fn get_resource_sequence(
    client: &mut sqlx::PgConnection,
    tenant_id: &str,
    cur_sequence: i64,
    count: Option<u64>,
) -> Result<Vec<ReturnV>, OperationOutcomeError> {
    let result = query_as!(
        ReturnV,
        r#"SELECT id, fhir_method as "fhir_method: FHIRMethod", sequence, resource as "resource: FHIRJson<Resource>" FROM resources WHERE tenant = $1 AND sequence > $2 ORDER BY sequence LIMIT $3 "#,
        tenant_id,
        cur_sequence,
        count.unwrap_or(100) as i64
    )
    .fetch_all(client)
    .await
    .map_err(IndexingWorkerError::from)?;

    Ok(result)
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

pub async fn index_for_tenant() {}

pub async fn run_worker() {
    // Initialize the PostgreSQL connection pool
    let config = get_config("environment".into());
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let url = Url::parse("https://localhost:9200").unwrap();
    let conn_pool = SingleNodeConnectionPool::new(url);
    let transport = TransportBuilder::new(conn_pool)
        .cert_validation(CertificateValidation::None)
        .auth(Credentials::Basic(
            "elastic".to_string(),
            "nGN1wSIQ-8phdE*JiLOp".to_string(),
        ))
        .build()
        .unwrap();
    let elasticsearch_client = Arc::new(Elasticsearch::new(transport));

    let indices_client = elasticsearch_client
        .indices()
        .create(IndicesCreateParts::Index(R4_FHIR_INDEX))
        .send()
        .await;

    tracing::info!("Indices create response: {:?}", indices_client);

    let mut pg_connection = sqlx::PgConnection::connect(&config.get("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to the database");
    let mut cursor = OffsetDateTime::UNIX_EPOCH;
    let tenants_limit: usize = 100;

    let fp_engine = Arc::new(oxidized_fhirpath::FPEngine::new());

    let patient_params = Arc::new(
        R4_SEARCH_PARAMETERS
            .values()
            .filter(|sp| {
                let code = sp
                    .base
                    .iter()
                    .filter_map(|b| b.value.as_ref().map(|v| v.as_str()))
                    .collect::<Vec<_>>();
                sp.expression.is_some()
                    && (code.contains(&"Patient")
                        || code.contains(&"Resource")
                        || code.contains(&"DomainResource"))
            })
            .collect::<Vec<_>>(),
    );

    loop {
        let tenants_to_check = get_tenants(&mut pg_connection, &cursor, tenants_limit).await;
        if let Err(error) = tenants_to_check {
            tracing::error!("Failed to retrieve tenants: {:?}", error);
            break;
        } else if let Ok(tenants_to_check) = tenants_to_check {
            if tenants_to_check.is_empty() || tenants_to_check.len() < tenants_limit {
                cursor = OffsetDateTime::UNIX_EPOCH; // Reset cursor if no tenants found
            } else {
                cursor = tenants_to_check[0].created_at;
            }

            for tenant in tenants_to_check {
                // sleep(Duration::from_millis(1000)).await;
                let tenant_id = tenant.id.clone();
                let start = Instant::now();
                let fp_engine = fp_engine.clone();
                let patient_params = patient_params.clone();
                let elasticsearch_client = elasticsearch_client.clone();
                let index_tenant_result: Result<(), IndexingWorkerError> = pg_connection
                    .transaction(|transaction| {
                        Box::pin(async move {
                            let tenant_lock_provider = PostgresIndexLockProvider::new();
                            let tenant_locks = tenant_lock_provider
                                .get_available(transaction, vec![&tenant_id])
                                .await?;

                            if tenant_locks.is_empty() {
                                return Ok(());
                            }

                            let resources = get_resource_sequence(
                                transaction,
                                "tenant",
                                tenant_locks[0].index_sequence_position,
                                Some(1000),
                            )
                            .await?;

                            tracing::info!("Available locks: {:?}", tenant_locks);
                            tracing::info!("Retrieved resources: {:?}", resources.len());

                            // Iterator used to evaluate all of the search expressions for indexing.
                            let bulk_ops: Vec<BulkOperation<HashMap<String, InsertableIndex>>> =
                                resources
                                    .par_iter()
                                    .map(|r| {
                                        let mut map = HashMap::new();
                                        for param in patient_params.iter() {
                                            if let Some(expression) = param
                                                .expression
                                                .as_ref()
                                                .and_then(|e| e.value.as_ref())
                                                && let Some(url) = param.url.value.as_ref()
                                            {
                                                let result = fp_engine
                                                    .evaluate(expression, vec![&r.resource.0])
                                                    .map_err(IndexingWorkerError::from)?;

                                                let result_vec = conversion::to_insertable_index(
                                                    param,
                                                    result.iter().collect::<Vec<_>>(),
                                                )?;

                                                map.insert(url.clone(), result_vec);
                                            }
                                        }
                                        let k: elasticsearch::BulkCreateOperation<
                                            HashMap<String, InsertableIndex>,
                                        > = BulkOperation::create(map)
                                            .id(&r.id)
                                            .index(R4_FHIR_INDEX);
                                        Ok(k.into())
                                        // map
                                    })
                                    .collect::<Result<Vec<_>, OperationOutcomeError>>()?;

                            elasticsearch_client
                                .bulk(BulkParts::Index(R4_FHIR_INDEX))
                                .body(bulk_ops)
                                .send()
                                .await?;

                            if let Some(resource) = resources.last() {
                                tenant_lock_provider
                                    .update(transaction, &tenant_id, resource.sequence as usize)
                                    .await?;
                            }

                            Ok(())
                        })
                    })
                    .await;

                if let Err(_error) = index_tenant_result {
                    tracing::error!(
                        "Failed to index tenant: '{}' cause: '{:?}'",
                        &tenant.id,
                        _error
                    );
                } else {
                    tracing::info!(
                        "Successfully indexed tenant: {} in {:?}",
                        &tenant.id,
                        start.elapsed()
                    );
                }
            }
        }
    }
}
