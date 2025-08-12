use crate::indexing_lock::{IndexLockProvider, postgres::PostgresIndexLockProvider};
use oxidized_config::get_config;
use oxidized_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use oxidized_fhir_repository::{
    FHIRRepository, FHIRTransaction, SupportedFHIRVersions, TenantId,
    postgres::{PostgresRepository, SQLImplementation},
};
use oxidized_fhir_search::{IndexResource, SearchEngine, elastic_search::ElasticSearchEngine};
use oxidized_fhirpath::FHIRPathError;
use sqlx::{Pool, Postgres, Transaction, query_as, types::time::OffsetDateTime};
use std::sync::Arc;

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

struct TenantReturn {
    id: TenantId,
    created_at: OffsetDateTime,
}

async fn get_tenants(
    client: &Pool<Postgres>,
    cursor: &OffsetDateTime,
    count: usize,
) -> Result<Vec<TenantReturn>, OperationOutcomeError> {
    let result = query_as!(
        TenantReturn,
        r#"SELECT id as "id: TenantId", created_at FROM tenants WHERE created_at > $1 ORDER BY created_at DESC LIMIT $2"#,
        cursor,
        count as i64
    )
    .fetch_all(client)
    .await
    .map_err(IndexingWorkerError::from)?;

    Ok(result)
}

async fn index_tenant_next_sequence<'a, Engine: SearchEngine + 'a>(
    search_client: Arc<Engine>,
    tx: &'a mut Transaction<'_, Postgres>,
    tenant_id: &'a TenantId,
) -> Result<(), IndexingWorkerError> {
    let tenant_lock_provider = PostgresIndexLockProvider::new();
    let tenant_locks = tenant_lock_provider
        .get_available(tx, vec![tenant_id.as_ref()])
        .await?;

    if tenant_locks.is_empty() {
        return Ok(());
    }

    let resources = SQLImplementation::get_sequence(
        &mut *tx,
        tenant_id,
        tenant_locks[0].index_sequence_position as u64,
        Some(1000),
    )
    .await?;

    if !resources.is_empty() {
        tracing::info!(
            "Tenant '{}' Indexing '{}' resources",
            tenant_id,
            resources.len()
        );
    }

    // Perform indexing if there are resources to index.
    if !resources.is_empty() {
        search_client
            .clone()
            .index(
                &SupportedFHIRVersions::R4,
                &tenant_id,
                resources
                    .iter()
                    .map(|r| IndexResource {
                        id: &r.id,
                        version_id: &r.version_id,
                        project: &r.project,
                        fhir_method: &r.fhir_method,
                        resource_type: &r.resource_type,
                        resource: &r.resource.0,
                    })
                    .collect(),
            )
            .await?;
        if let Some(resource) = resources.last() {
            tenant_lock_provider
                .update(tx, tenant_id.as_ref(), resource.sequence as usize)
                .await?;
        }
    }

    Ok(())
}

async fn index_for_tenant<
    Search: SearchEngine,
    Repository: FHIRRepository<Transaction = Transaction<'static, Postgres>>,
>(
    repo: Arc<Repository>,
    search_client: Arc<Search>,
    tenant_id: &TenantId,
) -> Result<(), IndexingWorkerError> {
    let search_client = search_client.clone();

    let mut tx = repo.transaction().await.unwrap();
    let res = index_tenant_next_sequence(search_client, &mut tx, &tenant_id).await;

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

    let search_engine = Arc::new(
        ElasticSearchEngine::new(
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
        )
        .expect("Failed to create Elasticsearch client"),
    );

    search_engine
        .migrate(&SupportedFHIRVersions::R4)
        .await
        .expect("Failed to create mapping for R4 index");

    let pg_pool = sqlx::PgPool::connect(&config.get("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to the database");
    let repo = Arc::new(PostgresRepository::new(pg_pool.clone()));
    let mut cursor = OffsetDateTime::UNIX_EPOCH;
    let tenants_limit: usize = 100;

    tracing::info!("Starting indexing worker...");

    loop {
        let tenants_to_check = get_tenants(&pg_pool, &cursor, tenants_limit).await;
        if let Ok(tenants_to_check) = tenants_to_check {
            if tenants_to_check.is_empty() || tenants_to_check.len() < tenants_limit {
                cursor = OffsetDateTime::UNIX_EPOCH; // Reset cursor if no tenants found
            } else {
                cursor = tenants_to_check[0].created_at;
            }

            for tenant in tenants_to_check {
                let result =
                    index_for_tenant(repo.clone(), search_engine.clone(), &tenant.id).await;

                if let Err(_error) = result {
                    tracing::error!(
                        "Failed to index tenant: '{}' cause: '{:?}'",
                        &tenant.id,
                        _error
                    );
                }
            }
        } else if let Err(error) = tenants_to_check {
            tracing::error!("Failed to retrieve tenants: {:?}", error);
        }
    }
}
