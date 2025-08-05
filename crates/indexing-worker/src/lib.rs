use crate::{
    conversion::InsertableIndex,
    indexing_lock::{IndexLockProvider, postgres::PostgresIndexLockProvider},
};
use elasticsearch::{BulkOperation, BulkParts};
use oxidized_config::get_config;
use oxidized_fhir_model::r4::types::{Resource, SearchParameter};
use oxidized_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use oxidized_fhir_repository::{
    FHIRMethod, FHIRRepository, FHIRTransaction, SupportedFHIRVersions, TenantId,
    postgres::{FHIRPostgresRepositoryPool, SQLImplementation},
};
use oxidized_fhir_search::{SearchEngine, elastic_search::ElasticSearchEngine};
use oxidized_fhirpath::{FHIRPathError, FPEngine};
use rayon::prelude::*;
use sqlx::{Pool, Postgres, Transaction, query_as, types::time::OffsetDateTime};
use std::{collections::HashMap, pin::Pin, sync::Arc, time::Instant};

mod conversion;
mod indexing_lock;

#[derive(OperationOutcomeError, Debug)]
pub enum IndexingWorkerError {
    #[fatal(code = "exception", diagnostic = "Database error: '{arg0}'")]
    DatabaseConnectionError(#[from] sqlx::Error),
    #[fatal(code = "exception", diagnostic = "Lock error: '{arg0}'")]
    OperationError(#[from] OperationOutcomeError),
    #[fatal(code = "exception", diagnostic = "Elasticsearch error: '{arg0}'")]
    ElasticsearchError(#[from] elasticsearch::Error),
    #[fatal(code = "exception", diagnostic = "FHIRPath error: '{arg0}'")]
    FHIRPathError(#[from] FHIRPathError),
    #[fatal(code = "exception", diagnostic = "Unsupported FHIR method: '{arg0}'")]
    UnsupportedFHIRMethod(FHIRMethod),
    #[fatal(
        code = "exception",
        diagnostic = "Missing search parameters for resource: '{arg0}'"
    )]
    MissingSearchParameters(String),
    #[fatal(
        code = "exception",
        diagnostic = "Fatal error occurred during indexing"
    )]
    Fatal,
    #[fatal(
        code = "exception",
        diagnostic = "Artifact error: Invalid resource type '{arg0}'"
    )]
    ResourceTypeError(#[from] oxidized_fhir_model::r4::types::ResourceTypeError),
}

static R4_FHIR_INDEX: &str = "r4_search_index";

struct TenantReturn {
    id: String,
    created_at: OffsetDateTime,
}

async fn get_tenants(
    client: &Pool<Postgres>,
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

fn resource_to_elastic_index(
    fp_engine: Arc<FPEngine>,
    parameters: &Vec<Arc<SearchParameter>>,
    resource: &Resource,
) -> Result<HashMap<String, InsertableIndex>, OperationOutcomeError> {
    let mut map = HashMap::new();
    for param in parameters.iter() {
        if let Some(expression) = param.expression.as_ref().and_then(|e| e.value.as_ref())
            && let Some(url) = param.url.value.as_ref()
        {
            let result = fp_engine
                .evaluate(expression, vec![resource])
                .map_err(IndexingWorkerError::from)?;

            let result_vec =
                conversion::to_insertable_index(param, result.iter().collect::<Vec<_>>())?;

            map.insert(url.clone(), result_vec);
        }
    }

    Ok(map)
}

fn index_tenant_next_sequence(
    tx: &mut Transaction<'_, Postgres>,
    fp_engine: Arc<FPEngine>,
    tenant_id: &TenantId,
) -> Pin<Box<dyn Future<Output = Result<(), IndexingWorkerError>> + Send>> {
    Box::pin(async move {
        let tenant_lock_provider = PostgresIndexLockProvider::new();
        let tenant_locks = tenant_lock_provider
            .get_available(tx, vec![tenant_id.as_ref()])
            .await?;

        if tenant_locks.is_empty() {
            return Ok(());
        }

        let resources = SQLImplementation::get_sequence(
            tx,
            tenant_id,
            tenant_locks[0].index_sequence_position as u64,
            Some(1000),
        )
        .await?;

        tracing::info!("Available locks: {:?}", tenant_locks);
        tracing::info!("Retrieved resources: {:?}", resources.len());

        // Iterator used to evaluate all of the search expressions for indexing.
        let bulk_ops: Vec<BulkOperation<HashMap<String, InsertableIndex>>> = resources
            .par_iter()
            .filter(|r| match r.fhir_method {
                FHIRMethod::Create | FHIRMethod::Update | FHIRMethod::Delete => true,
                _ => false,
            })
            .map(|r| match &r.fhir_method {
                FHIRMethod::Create | FHIRMethod::Update => {
                    let params =
                        oxidized_artifacts::search_parameters::get_search_parameters_for_resource(
                            &r.resource_type,
                        )
                        .map_err(IndexingWorkerError::from)?;
                    let mut elastic_index =
                        resource_to_elastic_index(fp_engine.clone(), &params, &r.resource.0)?;

                    elastic_index.insert(
                        "resource_type".to_string(),
                        InsertableIndex::String(vec![r.resource_type.clone()]),
                    );
                    elastic_index.insert(
                        "version_id".to_string(),
                        InsertableIndex::String(vec![r.version_id.clone()]),
                    );
                    elastic_index.insert(
                        "project".to_string(),
                        InsertableIndex::String(vec![r.project.clone()]),
                    );
                    elastic_index.insert(
                        "tenant".to_string(),
                        InsertableIndex::String(vec![r.tenant.clone()]),
                    );

                    Ok(BulkOperation::index(elastic_index)
                        .id(&r.id)
                        .index(R4_FHIR_INDEX)
                        .into())
                }
                FHIRMethod::Delete => Ok(BulkOperation::delete(&r.id).index(R4_FHIR_INDEX).into()),
                method => Err(IndexingWorkerError::UnsupportedFHIRMethod(method.clone()).into()),
            })
            .collect::<Result<Vec<_>, OperationOutcomeError>>()?;

        if !bulk_ops.is_empty() {
            let res = elasticsearch_client
                .bulk(BulkParts::Index(R4_FHIR_INDEX))
                .body(bulk_ops)
                .send()
                .await?;

            if !res.status_code().is_success() {
                tracing::error!(
                    "Failed to index resources for tenant: '{}'. Response: '{:?}', body: '{}'",
                    tenant_id,
                    res.status_code(),
                    res.text().await.unwrap()
                );
                return Err(IndexingWorkerError::Fatal);
            }

            if let Some(resource) = resources.last() {
                tenant_lock_provider
                    .update(tx, tenant_id.as_ref(), resource.sequence as usize)
                    .await?;
            }
        }

        Ok(())
    })
}

async fn index_for_tenant<
    Search: SearchEngine,
    Repository: FHIRRepository<Transaction = Transaction<'static, Postgres>>,
>(
    repo: Arc<Repository>,
    elasticsearch_client: Arc<Search>,
    fp_engine: Arc<FPEngine>,
    tenant_id: String,
) -> Result<(), IndexingWorkerError> {
    let fp_engine = fp_engine.clone();
    let elasticsearch_client = elasticsearch_client.clone();

    let mut tx = repo.transaction().await.unwrap();
    let res = index_tenant_next_sequence(&mut tx, fp_engine, &TenantId::new(tenant_id)).await;

    match res {
        Ok(res) => {
            tx.commit().await?;
            Ok(res)
        }
        Err(e) => {
            tx.rollback().await?;
            Err(e)
        }
    }
}

pub async fn run_worker() {
    let config = get_config("environment".into());
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).unwrap();
    let fp_engine = Arc::new(oxidized_fhirpath::FPEngine::new());

    let search_engine = Arc::new(ElasticSearchEngine::new(
        fp_engine.clone(),
        &config
            .get("ELASTICSEARCH_URL")
            .expect("ELASTICSEARCH_URL variable not set"),
        config
            .get("ELASTICSEARCH_USERNAME")
            .expect("ELASTICSEARCH_USERNAME variable not set"),
        config
            .get("ELASTICSEARCH_PASSWORD")
            .expect("ELASTICSEARCH_PASSWORD variable not set"),
    ));

    search_engine
        .migrate(SupportedFHIRVersions::R4, R4_FHIR_INDEX)
        .await
        .expect("Failed to create mapping for R4 index");

    let mut pg_pool = sqlx::PgPool::connect(&config.get("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to the database");
    let repo = Arc::new(FHIRPostgresRepositoryPool::new(pg_pool.clone()));
    let mut cursor = OffsetDateTime::UNIX_EPOCH;
    let tenants_limit: usize = 100;

    loop {
        let tenants_to_check = get_tenants(&pg_pool, &cursor, tenants_limit).await;
        if let Ok(tenants_to_check) = tenants_to_check {
            if tenants_to_check.is_empty() || tenants_to_check.len() < tenants_limit {
                cursor = OffsetDateTime::UNIX_EPOCH; // Reset cursor if no tenants found
            } else {
                cursor = tenants_to_check[0].created_at;
            }

            for tenant in tenants_to_check {
                let start = Instant::now();
                let result = index_for_tenant(
                    repo.clone(),
                    search_engine.clone(),
                    fp_engine.clone(),
                    tenant.id.clone(),
                )
                .await;

                if let Err(_error) = result {
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
        } else if let Err(error) = tenants_to_check {
            tracing::error!("Failed to retrieve tenants: {:?}", error);
        }
    }
}
