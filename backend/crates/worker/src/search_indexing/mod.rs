use crate::{
    indexing_lock::{IndexLockProvider, postgres::ProjectLockIndex},
    traits::Worker,
};
use haste_fhir_model::r4::generated::resources::ResourceTypeError;
use haste_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use haste_fhir_search::config::{SearchConfig, SearchEngineBackend};
use haste_fhir_search::{IndexResource, SearchEngine};
use haste_fhirpath::FHIRPathError;
use haste_jwt::{ProjectId, TenantId, VersionId};
use haste_repository::config::{RepoConfig, create_repo};
use haste_repository::{
    failed_indexing::{FailedIndexRecord, FailedIndexingProvider},
    fhir::FHIRRepository,
    pg::PGConnection,
    sequence::{ResourcePollingValue, ResourceSequential},
    types::{SearchIndexBackend, SupportedFHIRVersions},
};
use serde::{Deserialize, Serialize};
use sqlx::{Acquire, query_as, types::time::OffsetDateTime};
use std::{sync::Arc, time::Instant};
use tokio::{
    sync::{Mutex, Semaphore},
    task::JoinHandle,
};

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

struct ProjectReturn {
    tenant: TenantId,
    project: ProjectId,
    created_at: OffsetDateTime,
}

async fn get_projects(
    repo: &PGConnection,
    cursor: &OffsetDateTime,
    count: i64,
) -> Result<Vec<ProjectReturn>, OperationOutcomeError> {
    match repo {
        PGConnection::Pool(pool, _) => {
            let mut connection = pool.acquire().await.map_err(IndexingWorkerError::from)?;
            let conn = connection
                .acquire()
                .await
                .map_err(IndexingWorkerError::from)?;
            let result = query_as!(
                ProjectReturn,
                r#"SELECT tenant as "tenant: TenantId", id as "project: ProjectId", created_at FROM projects WHERE created_at > $1 ORDER BY created_at DESC LIMIT $2"#,
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
                ProjectReturn,
                r#"SELECT tenant as "tenant: TenantId", id as "project: ProjectId", created_at FROM projects WHERE created_at > $1 ORDER BY created_at DESC LIMIT $2"#,
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

/// Records in `FailedIndexingProvider` if any resources failed to index. This is a no-op if the list is empty.
async fn record_failures(
    tenant: &TenantId,
    project: &ProjectId,
    indexing_error_provider: &impl FailedIndexingProvider,
    failures: &[FailedIndexRecord],
) -> Result<(), IndexingWorkerError> {
    if failures.is_empty() {
        return Ok(());
    }

    tracing::warn!(
        "Parking {} resource(s) that failed indexing for tenant '{}' project '{}'.",
        failures.len(),
        tenant,
        project.as_ref()
    );

    for failure in failures {
        tracing::error!(
            tenant = %tenant,
            project = %project.as_ref(),
            resource_type = %failure.resource_type,
            version_id = failure.version_id.as_ref(),
            fhir_method = failure.fhir_method.as_str(),
            error = %failure.error_message,
            "Search indexing failed for resource"
        );
    }

    indexing_error_provider.record_failures(failures).await?;

    Ok(())
}

async fn update_lock_sequence_position<
    Repo: ResourceSequential + IndexLockProvider<(TenantId, ProjectId), ProjectLockIndex>,
>(
    tenant_id: &TenantId,
    project_id: &ProjectId,
    backend: SearchIndexBackend,
    repo: &Repo,
    start_sequence: Option<i64>,
    resources_total: usize,
    start: Instant,
    last_polling_value: ResourcePollingValue,
) -> Result<(), OperationOutcomeError> {
    let diff = (last_polling_value.sequence + 1) - start_sequence.unwrap_or(0);
    let total = resources_total;

    if total as u64 != diff.unsigned_abs() {
        tracing::event!(
            tracing::Level::WARN,
            // safe_seq = resource.max_safe_seq.unwrap_or(0),
            first_seq = start_sequence.unwrap_or(0),
            last_seq = last_polling_value.sequence,
            total = resources_total,
            diff = (last_polling_value.sequence + 1) - start_sequence.unwrap_or(0),
            "Sequence gap detected while indexing tenant '{}' project '{}' ({}) - resources may have been skipped.",
            tenant_id,
            project_id.as_ref(),
            backend
        );
    }

    tracing::trace!(
        "Updating {} lock for tenant '{}' project '{}' to sequence position {}.",
        backend,
        tenant_id,
        project_id.as_ref(),
        last_polling_value.sequence
    );

    repo.update_lock(
        backend,
        &(tenant_id.clone(), project_id.clone()),
        ProjectLockIndex {
            tenant: tenant_id.clone(),
            project: project_id.clone(),
            index_sequence_position: last_polling_value.sequence,
        },
    )
    .await?;

    tracing::trace!(
        "Indexed {} resources for tenant '{}' project '{}' in {:.2?} (up to sequence {})",
        resources_total,
        tenant_id.as_ref(),
        project_id.as_ref(),
        start.elapsed(),
        last_polling_value.sequence
    );

    Ok(())
}

static TOTAL_INDEXED: std::sync::LazyLock<Mutex<usize>> =
    std::sync::LazyLock::new(|| Mutex::new(0));

async fn index_project_next_sequence<
    Repo: ResourceSequential
        + IndexLockProvider<(TenantId, ProjectId), ProjectLockIndex>
        + FailedIndexingProvider,
    Engine: SearchEngine,
>(
    max_concurrent_limit: u64,
    backend: SearchIndexBackend,
    search_client: Arc<Engine>,
    repo: &Repo,
    tenant_id: &TenantId,
    project_id: &ProjectId,
) -> Result<(), IndexingWorkerError> {
    let start = std::time::Instant::now();
    let lock_key = (tenant_id.clone(), project_id.clone());
    let project_locks = repo.get_available_locks(backend, vec![&lock_key]).await?;

    if project_locks.is_empty() {
        tracing::info!(
            "No available {} lock for tenant '{}' project '{}', skipping indexing.",
            backend,
            tenant_id,
            project_id.as_ref()
        );
        return Ok(());
    }

    tracing::trace!(
        "Acquired {} lock for tenant '{}' project '{}', starting indexing from sequence {}.",
        backend,
        tenant_id,
        project_id.as_ref(),
        project_locks[0].index_sequence_position
    );

    let resources = repo
        .get_sequence(
            tenant_id,
            project_id,
            project_locks[0].index_sequence_position.cast_unsigned(),
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
                        sequence: r.sequence,
                    })
                    .collect(),
            )
            .await?;
        let resources_attempted_to_index_count = outcome.succeeded + outcome.failed.len();

        if resources_attempted_to_index_count != resources_total {
            tracing::error!(
                "Indexed+failed resource count '{}' does not match retrieved resource count '{}' for tenant '{}' project '{}'",
                resources_attempted_to_index_count,
                resources_total,
                tenant_id,
                project_id.as_ref()
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

        record_failures(tenant_id, project_id, repo, &failures).await?;

        if let Some(last_polling_value) = last_value {
            update_lock_sequence_position(
                tenant_id,
                project_id,
                backend,
                repo,
                start_sequence,
                resources_total,
                start,
                last_polling_value,
            )
            .await?;
        }

        *(TOTAL_INDEXED.lock().await) += outcome.succeeded;
    }

    Ok(())
}

async fn index_for_project<
    Search: SearchEngine,
    Repository: FHIRRepository
        + ResourceSequential
        + IndexLockProvider<(TenantId, ProjectId), ProjectLockIndex>
        + FailedIndexingProvider,
>(
    max_concurrent_limit: u64,
    backend: SearchIndexBackend,
    repo: Arc<Repository>,
    search_client: Arc<Search>,
    tenant_id: &TenantId,
    project_id: &ProjectId,
) -> Result<(), IndexingWorkerError> {
    let tx = repo
        .transaction(false)
        .await
        .map_err(IndexingWorkerError::from)?;
    let res = index_project_next_sequence(
        max_concurrent_limit,
        backend,
        search_client,
        &tx,
        tenant_id,
        project_id,
    )
    .await;

    match res {
        Ok(res) => {
            tx.commit().await?;
            Ok(res)
        }
        Err(e) => {
            if let Err(rollback_err) = tx.rollback().await {
                tracing::error!(
                    "Failed to roll back transaction for tenant '{}' project '{}' (original error: '{:?}'): '{:?}'",
                    tenant_id,
                    project_id.as_ref(),
                    e,
                    rollback_err
                );
                return Err(rollback_err.into());
            }
            Err(e)
        }
    }
}

pub struct IndexingWorker {
    max_concurrent_limit: Option<u64>,
    tenant_concurrency: Option<u64>,
    running: Arc<tokio::sync::Mutex<bool>>,
    repo: Arc<PGConnection>,
    search_engine: Arc<SearchEngineBackend>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct WorkerEnvironment {
    pub max_concurrent_limit: Option<u64>,
    /// Maximum number of projects indexed concurrently within one poll.
    /// Field name kept as `tenant_concurrency` for config compatibility -
    /// locking moved from tenant- to project-level, but this still bounds
    /// the same underlying semaphore.
    pub tenant_concurrency: Option<u64>,
    pub repo: RepoConfig,
    pub search: SearchConfig,
}

impl Default for WorkerEnvironment {
    fn default() -> Self {
        Self {
            max_concurrent_limit: Some(1000),
            // Matches `PostgresRepoConfig::default`'s `max_connections`.
            tenant_concurrency: Some(10),
            repo: RepoConfig::default(),
            search: SearchConfig::default(),
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

        // This worker only writes documents, so it is never allowed to change
        // the index's shape out from under a running server.
        let search_engine =
            haste_fhir_search::config::create_search_engine(&config.search, repo.clone(), false)
                .await?;

        let mut attempts = 0;
        while search_engine.is_connected().await.is_err() && attempts < 5 {
            tracing::error!(
                "{} is not connected, retrying in 5 seconds...",
                search_engine.name()
            );
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            attempts += 1;
        }

        search_engine.is_connected().await?;

        Ok(Self {
            max_concurrent_limit: config.max_concurrent_limit,
            tenant_concurrency: config.tenant_concurrency,
            running: Arc::new(tokio::sync::Mutex::new(true)),
            repo,
            search_engine,
        })
    }
}

impl Worker for IndexingWorker {
    async fn run(&self) -> Result<JoinHandle<()>, OperationOutcomeError> {
        let mut cursor = OffsetDateTime::UNIX_EPOCH;
        let projects_limit: u64 = 100;

        tracing::info!("Starting indexing worker...");

        let mut k = *TOTAL_INDEXED.lock().await;

        let repo = self.repo.clone();
        let search_engine: Arc<SearchEngineBackend> = self.search_engine.clone();
        let backend = search_engine.lock_kind();
        let running = self.running.clone();
        let max_concurrent_limit = self.max_concurrent_limit.unwrap_or(1000);
        let project_semaphore = Arc::new(Semaphore::new(
            usize::try_from(self.tenant_concurrency.unwrap_or(10).max(1)).unwrap_or(usize::MAX),
        ));

        let spawned = tokio::spawn(async move {
            while *running.lock().await {
                let projects_to_check =
                    get_projects(repo.as_ref(), &cursor, projects_limit.cast_signed()).await;

                // Nothing to do this iteration (no projects, or the fetch itself
                // failed) - back off instead of hammering Postgres in a tight spin.
                let idle = match projects_to_check {
                    Ok(projects_to_check) => {
                        let idle = projects_to_check.is_empty();
                        if idle || (projects_to_check.len() as u64) < projects_limit {
                            cursor = OffsetDateTime::UNIX_EPOCH; // Reset cursor if no projects found
                        } else {
                            cursor = projects_to_check[0].created_at;
                        }

                        let handles: Vec<_> = projects_to_check
                            .into_iter()
                            .map(|project| {
                                let repo = repo.clone();
                                let search_engine = search_engine.clone();
                                let semaphore = project_semaphore.clone();

                                tokio::spawn(async move {
                                    let _permit = match semaphore.acquire_owned().await {
                                        Ok(permit) => permit,
                                        Err(_closed) => {
                                            tracing::warn!(
                                                "Project semaphore closed; skipping indexing for tenant '{}' project '{}'.",
                                                &project.tenant,
                                                project.project.as_ref()
                                            );
                                            return;
                                        }
                                    };

                                    tracing::trace!(
                                        "Indexing tenant: '{}' project: '{}'",
                                        &project.tenant,
                                        project.project.as_ref()
                                    );

                                    // Boxed: the search engine's indexing future is
                                    // large, and this one is spawned per project.
                                    let result = Box::pin(index_for_project(
                                        max_concurrent_limit,
                                        backend,
                                        repo,
                                        search_engine,
                                        &project.tenant,
                                        &project.project,
                                    ))
                                    .await;

                                    if let Err(error) = result {
                                        tracing::error!(
                                            "Failed to index tenant: '{}' project: '{}' cause: '{:?}'",
                                            &project.tenant,
                                            project.project.as_ref(),
                                            error
                                        );
                                    }
                                })
                            })
                            .collect();

                        for handle in handles {
                            if let Err(join_error) = handle.await {
                                tracing::error!(
                                    "Project indexing task panicked: '{:?}'",
                                    join_error
                                );
                            }
                        }

                        idle
                    }
                    Err(error) => {
                        tracing::error!("Failed to retrieve projects: {:?}", error);
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
