use crate::indexing_lock::IndexLockProvider;
use oxidized_config::get_config;
use oxidized_fhir_model::r4::generated::resources::ResourceTypeError;
use oxidized_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use oxidized_fhir_search::{IndexResource, SearchEngine, elastic_search::ElasticSearchEngine};
use oxidized_fhirpath::FHIRPathError;
use oxidized_jwt::TenantId;
use oxidized_repository::{
    fhir::{FHIRRepository, IsolationLevel},
    types::SupportedFHIRVersions,
};
use sqlx::{Pool, Postgres, query_as, types::time::OffsetDateTime};
use std::sync::Arc;
use tokio::sync::Mutex;

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
    ResourceTypeError(#[from] ResourceTypeError),
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

static TOTAL_INDEXED: std::sync::LazyLock<Mutex<usize>> =
    std::sync::LazyLock::new(|| Mutex::new(0));

async fn index_tenant_next_sequence<
    Repo: FHIRRepository + IndexLockProvider,
    Engine: SearchEngine,
>(
    search_client: Arc<Engine>,
    tx: &Repo,
    repo: &Repo,
    tenant_id: &TenantId,
) -> Result<(), IndexingWorkerError> {
    let start = std::time::Instant::now();
    let tenant_locks = tx.get_available_locks(vec![tenant_id]).await?;

    if tenant_locks.is_empty() {
        return Ok(());
    }

    let resources = tx
        .get_sequence(
            tenant_id,
            tenant_locks[0].index_sequence_position as u64,
            Some(1000),
        )
        .await?;

    // Perform indexing if there are resources to index.
    if !resources.is_empty() {
        let result = search_client
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

        if result.0 != resources.len() {
            tracing::error!(
                "Indexed resource count '{}' does not match retrieved resource count '{}'",
                result.0,
                resources.len()
            );
            return Err(IndexingWorkerError::Fatal);
        }

        if let Some(resource) = resources.last() {
            // println!(
            //     "safe_seq: {} first_seq: {} -> last_seq: {} <total: {}, sequence_diff: {}>",
            //     resource.max_safe_seq.unwrap_or(0),
            //     resources[0].sequence,
            //     resource.sequence,
            //     resources.len(),
            //     resource.sequence - resources[0].sequence
            // );

            let diff = (resource.sequence + 1) - resources[0].sequence;
            let total = resources.len();

            if total != diff as usize {
                tracing::event!(
                    tracing::Level::INFO,
                    // safe_seq = resource.max_safe_seq.unwrap_or(0),
                    first_seq = resources[0].sequence,
                    last_seq = resource.sequence,
                    total = resources.len(),
                    diff = (resource.sequence + 1) - resources[0].sequence
                );
            }

            tx.update_lock(tenant_id.as_ref(), resource.sequence as usize)
                .await?;
            // get the id of the last resource indexed
            // tracing::info!(
            //     "LAST RESOURCE INDEXED {} {:#?} ",
            //     resource.sequence,
            //     resource.resource.0
            // );
        }

        *(TOTAL_INDEXED.lock().await) += result.0;
    }

    Ok(())
}

async fn index_for_tenant<Search: SearchEngine, Repository: FHIRRepository + IndexLockProvider>(
    repo: Arc<Repository>,
    search_client: Arc<Search>,
    tenant_id: &TenantId,
) -> Result<(), IndexingWorkerError> {
    let search_client = search_client.clone();

    let tx = repo
        .transaction(Some(&IsolationLevel::ReadCommitted), false)
        .await
        .unwrap();

    let res = index_tenant_next_sequence(search_client, &tx, &*repo, &tenant_id).await;

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

pub enum IndexingWorkerEnvironmentVariables {
    DatabaseURL,
    ElasticSearchURL,
    ElasticSearchUsername,
    ElasticSearchPassword,
}

impl From<IndexingWorkerEnvironmentVariables> for String {
    fn from(value: IndexingWorkerEnvironmentVariables) -> Self {
        match value {
            IndexingWorkerEnvironmentVariables::DatabaseURL => "DATABASE_URL".to_string(),
            IndexingWorkerEnvironmentVariables::ElasticSearchURL => "ELASTICSEARCH_URL".to_string(),
            IndexingWorkerEnvironmentVariables::ElasticSearchUsername => {
                "ELASTICSEARCH_USERNAME".to_string()
            }
            IndexingWorkerEnvironmentVariables::ElasticSearchPassword => {
                "ELASTICSEARCH_PASSWORD".to_string()
            }
        }
    }
}

pub async fn run_worker() -> Result<(), OperationOutcomeError> {
    let config = get_config::<IndexingWorkerEnvironmentVariables>("environment".into());
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).unwrap();
    let fp_engine = Arc::new(oxidized_fhirpath::FPEngine::new());

    let search_engine = Arc::new(
        ElasticSearchEngine::new(
            fp_engine.clone(),
            &config
                .get(IndexingWorkerEnvironmentVariables::ElasticSearchURL)
                .expect(&format!(
                    "'{}' variable not set",
                    String::from(IndexingWorkerEnvironmentVariables::ElasticSearchURL)
                )),
            config
                .get(IndexingWorkerEnvironmentVariables::ElasticSearchUsername)
                .expect(&format!(
                    "'{}' variable not set",
                    String::from(IndexingWorkerEnvironmentVariables::ElasticSearchUsername)
                )),
            config
                .get(IndexingWorkerEnvironmentVariables::ElasticSearchPassword)
                .expect(&format!(
                    "'{}' variable not set",
                    String::from(IndexingWorkerEnvironmentVariables::ElasticSearchPassword)
                )),
        )
        .expect("Failed to create Elasticsearch client"),
    );

    search_engine
        .migrate(&SupportedFHIRVersions::R4)
        .await
        .expect("Failed to create mapping for R4 index");

    let pg_pool = sqlx::PgPool::connect(
        &config
            .get(IndexingWorkerEnvironmentVariables::DatabaseURL)
            .unwrap(),
    )
    .await
    .expect("Failed to connect to the database");

    let repo = Arc::new(oxidized_repository::pg::PGConnection::pool(pg_pool.clone()));
    let mut cursor = OffsetDateTime::UNIX_EPOCH;
    let tenants_limit: usize = 100;

    tracing::info!("Starting indexing worker...");

    let mut k = *TOTAL_INDEXED.lock().await;

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

        if k != *TOTAL_INDEXED.lock().await {
            k = *TOTAL_INDEXED.lock().await;
            tracing::info!("TOTAL INDEXED SO FAR: {}", k);
        }
    }
}
