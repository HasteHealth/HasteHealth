use crate::{
    failed_indexing::{FailedIndexRecord, FailedIndexingProvider},
    indexing_lock::{IndexLockProvider, postgres::TenantLockIndex},
    traits::Worker,
};
use haste_fhir_model::r4::generated::resources::ResourceTypeError;
use haste_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use haste_fhir_search::{
    IndexResource, SearchEngine,
    elastic_search::{
        ElasticSearchEngine, create_es_client,
        search_parameter_resolver::ElasticSearchParameterResolver,
    },
};
use haste_fhirpath::FHIRPathError;
use haste_jwt::{TenantId, VersionId};
use haste_repository::{
    fhir::FHIRRepository, pg::PGConnection, sequence::ResourceSequential,
    types::SupportedFHIRVersions,
};
use serde::{Deserialize, Serialize};
use sqlx::{Acquire, query_as, types::time::OffsetDateTime};
use std::sync::Arc;
use tokio::{sync::Mutex, task::JoinHandle};

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
    repo: &PGConnection,
    cursor: &OffsetDateTime,
    count: i64,
) -> Result<Vec<TenantReturn>, OperationOutcomeError> {
    match repo {
        PGConnection::Pool(pool, _) => {
            let mut connection = pool.acquire().await.map_err(IndexingWorkerError::from)?;
            let conn = connection
                .acquire()
                .await
                .map_err(IndexingWorkerError::from)?;
            let result = query_as!(
                TenantReturn,
                r#"SELECT id as "id: TenantId", created_at FROM tenants WHERE created_at > $1 ORDER BY created_at DESC LIMIT $2"#,
                cursor,
                count
            )
            .fetch_all(&mut *conn)
            .await
            .map_err(IndexingWorkerError::from)?;

            Ok(result)
        }
        PGConnection::Transaction(tx, _, _) => {
            let mut connection = tx.lock().await;
            let conn = connection
                .acquire()
                .await
                .map_err(IndexingWorkerError::from)?;
            let result = query_as!(
                TenantReturn,
                r#"SELECT id as "id: TenantId", created_at FROM tenants WHERE created_at > $1 ORDER BY created_at DESC LIMIT $2"#,
                cursor,
                count
            )
            .fetch_all(&mut *conn)
            .await
            .map_err(IndexingWorkerError::from)?;

            Ok(result)
        }
    }
}

/// Records in FailedIndexingProvider if any resources failed to index. This is a no-op if the list is empty.
async fn record_failures(
    tenant: &TenantId,
    indexing_error_provider: &impl FailedIndexingProvider,
    failures: &[FailedIndexRecord],
) -> Result<(), IndexingWorkerError> {
    if failures.is_empty() {
        return Ok(());
    }

    tracing::warn!(
        "Parking {} resource(s) that failed indexing for tenant '{}'.",
        failures.len(),
        tenant
    );

    indexing_error_provider.record_failures(&failures).await?;

    Ok(())
}

static TOTAL_INDEXED: std::sync::LazyLock<Mutex<usize>> =
    std::sync::LazyLock::new(|| Mutex::new(0));

async fn index_tenant_next_sequence<
    Repo: ResourceSequential + IndexLockProvider<TenantId, TenantLockIndex> + FailedIndexingProvider,
    Engine: SearchEngine,
>(
    max_concurrent_limit: u64,
    search_client: Arc<Engine>,
    repo: &Repo,
    tenant_id: &TenantId,
) -> Result<(), IndexingWorkerError> {
    let start = std::time::Instant::now();
    let tenant_locks = repo.get_available_locks(vec![tenant_id]).await?;

    if tenant_locks.is_empty() {
        tracing::info!(
            "No available locks for tenant '{}', skipping indexing.",
            tenant_id
        );
        return Ok(());
    }

    tracing::info!(
        "Acquired lock for tenant '{}', starting indexing from sequence {}.",
        tenant_id,
        tenant_locks[0].index_sequence_position
    );

    let resources = repo
        .get_sequence(
            tenant_id,
            tenant_locks[0].index_sequence_position.cast_unsigned(),
            Some(max_concurrent_limit),
        )
        .await?;

    let resources_total = resources.len();
    let start_sequence = resources.first().map(|r| r.sequence);
    let last_value = resources.last().cloned();

    // Perform indexing if there are resources to index.
    if !resources.is_empty() {
        let outcome = search_client
            .index(
                SupportedFHIRVersions::R4,
                resources
                    .into_iter()
                    .map(|r| IndexResource {
                        tenant: r.tenant,
                        id: r.id,
                        version_id: VersionId::new(r.version_id),
                        project: r.project,
                        fhir_method: r.fhir_method,
                        resource_type: r.resource_type,
                        resource: r.resource.0,
                    })
                    .collect(),
            )
            .await?;
        let resources_attempted_to_index_count = outcome.succeeded + outcome.failed.len();

        if resources_attempted_to_index_count != resources_total {
            tracing::error!(
                "Indexed+failed resource count '{}' does not match retrieved resource count '{}' for tenant '{}'",
                resources_attempted_to_index_count,
                resources_total,
                tenant_id
            );
            return Err(IndexingWorkerError::Fatal);
        }

        let failures = outcome
            .failed
            .into_iter()
            .map(|failure| FailedIndexRecord {
                tenant: failure.resource.tenant,
                project: failure.resource.project,
                version_id: failure.resource.version_id,
                resource_type: failure.resource.resource_type.as_ref().to_string(),
                fhir_method: failure.resource.fhir_method,
                error_message: failure.error.to_string(),
            })
            .collect::<Vec<_>>();

        record_failures(tenant_id, repo, &failures).await?;

        if let Some(resource) = last_value {
            let diff = (resource.sequence + 1) - start_sequence.unwrap_or(0);
            let total = resources_total;

            if total as u64 != diff.unsigned_abs() {
                tracing::event!(
                    tracing::Level::WARN,
                    // safe_seq = resource.max_safe_seq.unwrap_or(0),
                    first_seq = start_sequence.unwrap_or(0),
                    last_seq = resource.sequence,
                    total = resources_total,
                    diff = (resource.sequence + 1) - start_sequence.unwrap_or(0),
                    "Sequence gap detected while indexing tenant '{}' - resources may have been skipped.",
                    tenant_id
                );
            }

            tracing::trace!(
                "Updating lock for tenant '{}' to sequence position {}.",
                tenant_id,
                resource.sequence
            );

            repo.update_lock(
                tenant_id,
                TenantLockIndex {
                    id: tenant_id.clone(),
                    index_sequence_position: resource.sequence,
                },
            )
            .await?;

            let elapsed = start.elapsed();

            tracing::trace!(
                "Indexed {} resources for tenant '{}' in {:.2?} (up to sequence {})",
                outcome.succeeded,
                tenant_id.as_ref(),
                elapsed,
                resource.sequence
            );
        }

        *(TOTAL_INDEXED.lock().await) += outcome.succeeded;
    }

    Ok(())
}

async fn index_for_tenant<
    Search: SearchEngine,
    Repository: FHIRRepository
        + ResourceSequential
        + IndexLockProvider<TenantId, TenantLockIndex>
        + FailedIndexingProvider,
>(
    max_concurrent_limit: u64,
    repo: Arc<Repository>,
    search_client: Arc<Search>,
    tenant_id: &TenantId,
) -> Result<(), IndexingWorkerError> {
    let tx = repo
        .transaction(false)
        .await
        .map_err(IndexingWorkerError::from)?;
    let res = index_tenant_next_sequence(max_concurrent_limit, search_client, &tx, tenant_id).await;

    match res {
        Ok(res) => {
            tx.commit().await?;
            Ok(res)
        }
        Err(e) => {
            if let Err(rollback_err) = tx.rollback().await {
                tracing::error!(
                    "Failed to roll back transaction for tenant '{}' (original error: '{:?}'): '{:?}'",
                    tenant_id,
                    e,
                    rollback_err
                );
                return Err(rollback_err.into());
            }
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

pub struct IndexingWorker {
    max_concurrent_limit: Option<u64>,
    running: Arc<tokio::sync::Mutex<bool>>,
    repo: Arc<PGConnection>,
    search_engine: Arc<ElasticSearchEngine<ElasticSearchParameterResolver<PGConnection>>>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct WorkerEnvironment {
    pub max_concurrent_limit: Option<u64>,
    pub repo: RepoConfig,
    pub search: SearchConfig,
}

// Repo backend where the FHIR server stores its data/resources.
#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum RepoConfig {
    Postgres(PostgresConfig),
}

#[derive(Clone, Deserialize, Serialize)]
pub struct PostgresConfig {
    pub database_url: String,
    pub max_connections: u32,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ElasticsearchConfig {
    pub url: String,
    pub username: String,
    pub password: String,
}

// Search backend where the FHIR server stores its search indices.
#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum SearchConfig {
    Elasticsearch(ElasticsearchConfig),
}

impl Default for WorkerEnvironment {
    fn default() -> Self {
        Self {
            max_concurrent_limit: Some(1000),
            repo: RepoConfig::default(),
            search: SearchConfig::default(),
        }
    }
}

impl Default for RepoConfig {
    fn default() -> Self {
        RepoConfig::Postgres(PostgresConfig::default())
    }
}
impl Default for PostgresConfig {
    fn default() -> Self {
        Self {
            database_url: "postgresql://postgres:postgres@localhost:5432/haste_health".into(),
            max_connections: 10,
        }
    }
}
impl Default for SearchConfig {
    fn default() -> Self {
        SearchConfig::Elasticsearch(ElasticsearchConfig::default())
    }
}
impl Default for ElasticsearchConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:9200".into(),
            username: "elastic".into(),
            password: "elastic".into(),
        }
    }
}

async fn create_repo(config: &RepoConfig) -> Result<Arc<PGConnection>, OperationOutcomeError> {
    match config {
        RepoConfig::Postgres(pg_config) => {
            let pool = sqlx::PgPool::connect(&pg_config.database_url)
                .await
                .map_err(IndexingWorkerError::from)?;
            Ok(Arc::new(PGConnection::pool(pool)))
        }
    }
}

fn create_search_engine(
    config: &SearchConfig,
    repo: &Arc<PGConnection>,
) -> Result<
    Arc<ElasticSearchEngine<ElasticSearchParameterResolver<PGConnection>>>,
    OperationOutcomeError,
> {
    match config {
        SearchConfig::Elasticsearch(elasticsearch_config) => {
            let es_client = create_es_client(
                &elasticsearch_config.url,
                elasticsearch_config.username.clone(),
                elasticsearch_config.password.clone(),
            )?;
            let search_engine = Arc::new(ElasticSearchEngine::new(
                Arc::new(ElasticSearchParameterResolver::new(
                    es_client.clone(),
                    repo.clone(),
                )),
                Arc::new(haste_fhirpath::FPEngine::new()),
                es_client,
            ));

            Ok(search_engine)
        }
    }
}

impl IndexingWorker {
    /// Creates and initializes a new worker.
    ///
    /// This initializes the repository and search engine using the provided
    /// configuration, then waits for the search engine to become available.
    /// The search engine connection is retried up to 5 times, with a 5-second
    /// delay between attempts.
    ///
    /// # Arguments
    ///
    /// * `config` - Shared worker configuration containing the repository,
    ///   search engine, and concurrency settings.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - creating the repository fails.
    /// - creating the search engine fails.
    /// - the search engine remains unavailable after 5 connection attempts.
    ///
    /// # Returns
    ///
    /// Returns an initialized [`Self`] with the repository and search engine
    /// ready for use.
    pub async fn new(config: Arc<WorkerEnvironment>) -> Result<Self, OperationOutcomeError> {
        let repo = create_repo(&config.repo).await?;
        let search_engine = create_search_engine(&config.search, &repo)?;

        let mut attempts = 0;
        while search_engine.is_connected().await.is_err() && attempts < 5 {
            tracing::error!("Elasticsearch is not connected, retrying in 5 seconds...");
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            attempts += 1;
        }

        if search_engine.is_connected().await.is_err() {
            return Err(OperationOutcomeError::fatal(
                haste_fhir_model::r4::generated::terminology::IssueType::exception(),
                "Elasticsearch is not connected after 5 attempts".to_string(),
            ));
        }

        Ok(Self {
            max_concurrent_limit: config.max_concurrent_limit,
            running: Arc::new(tokio::sync::Mutex::new(true)),
            repo,
            search_engine,
        })
    }
}

impl Worker for IndexingWorker {
    async fn run(&self) -> Result<JoinHandle<()>, OperationOutcomeError> {
        let mut cursor = OffsetDateTime::UNIX_EPOCH;
        let tenants_limit: u64 = 100;

        tracing::info!("Starting indexing worker...");

        let mut k = *TOTAL_INDEXED.lock().await;

        let repo = self.repo.clone();
        let search_engine: Arc<ElasticSearchEngine<ElasticSearchParameterResolver<PGConnection>>> =
            self.search_engine.clone();
        let running = self.running.clone();
        let max_concurrent_limit = self.max_concurrent_limit.unwrap_or(1000);

        let spawned = tokio::spawn(async move {
            while *running.lock().await {
                let tenants_to_check =
                    get_tenants(repo.as_ref(), &cursor, tenants_limit.cast_signed()).await;

                // Nothing to do this iteration (no tenants, or the fetch itself
                // failed) - back off instead of hammering Postgres in a tight spin.
                let idle = match tenants_to_check {
                    Ok(tenants_to_check) => {
                        let idle = tenants_to_check.is_empty();
                        if idle || (tenants_to_check.len() as u64) < tenants_limit {
                            cursor = OffsetDateTime::UNIX_EPOCH; // Reset cursor if no tenants found
                        } else {
                            cursor = tenants_to_check[0].created_at;
                        }

                        for tenant in tenants_to_check {
                            tracing::trace!("Indexing tenant: '{}'", &tenant.id);

                            let result = index_for_tenant(
                                max_concurrent_limit,
                                repo.clone(),
                                search_engine.clone(),
                                &tenant.id,
                            )
                            .await;

                            if let Err(error) = result {
                                tracing::error!(
                                    "Failed to index tenant: '{}' cause: '{:?}'",
                                    &tenant.id,
                                    error
                                );
                            }
                        }

                        idle
                    }
                    Err(error) => {
                        tracing::error!("Failed to retrieve tenants: {:?}", error);
                        true
                    }
                };

                if k != *TOTAL_INDEXED.lock().await {
                    k = *TOTAL_INDEXED.lock().await;
                    tracing::info!("TOTAL INDEXED SO FAR: {}", k);
                }

                if idle {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        });

        Ok(spawned)
    }

    async fn stop(&mut self) -> Result<(), OperationOutcomeError> {
        let mut running = self.running.lock().await;
        *running = false;
        Ok(())
    }
}
