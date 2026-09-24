//! Search backend configuration and construction.
//!
//! Lives here rather than in each binary so the server and the indexing worker
//! describe and build the same engine from the same settings. When they each
//! owned a copy, the worker's only knew about Elasticsearch — so pointing the
//! server at PostgreSQL left the worker indexing somewhere the server never
//! read, with nothing to signal it.

use std::sync::Arc;

use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhirpath::FPEngine;
use haste_repository::pg::PGConnection;
use serde::{Deserialize, Serialize};

use crate::{
    IndexOutcome, IndexResource, SearchEngine, SearchOptions, SearchReturn,
    elastic_search::{
        ElasticSearchEngine, create_es_client,
        search_parameter_resolver::ElasticSearchParameterResolver,
    },
    pg_search::{
        PgSearchEngine, create_pg_search_pool, search_parameter_resolver::PgSearchParameterResolver,
    },
};

/// Where the FHIR server stores its search index.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum SearchConfig {
    Elasticsearch(ElasticsearchConfig),
    Postgres(PostgresSearchConfig),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ElasticsearchConfig {
    pub url: String,
    pub username: String,
    pub password: String,
    /// Lets `migrate search` rebuild the index when a search parameter is
    /// removed, dropping its mapping and already-indexed data. Elasticsearch
    /// mappings are append-only, so dropping one needs a reindex. Off by
    /// default: removed parameters are logged, never deleted.
    #[serde(default)]
    pub prune_removed_search_parameters: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostgresSearchConfig {
    pub database_url: String,
    pub max_connections: u32,
}

impl Default for ElasticsearchConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:9200".to_string(),
            username: "elastic".to_string(),
            password: "elastic".to_string(),
            prune_removed_search_parameters: false,
        }
    }
}

impl Default for PostgresSearchConfig {
    fn default() -> Self {
        Self {
            database_url: "postgresql://postgres:postgres@127.0.0.1/haste_health_search"
                .to_string(),
            max_connections: 20,
        }
    }
}

impl Default for SearchConfig {
    fn default() -> Self {
        SearchConfig::Elasticsearch(ElasticsearchConfig::default())
    }
}

/// A configured search engine.
///
/// The concrete engines differ in their resolver type, so a caller holding one
/// generically would have to thread that type everywhere. This dispatches
/// instead, which is what lets the server and the worker switch backends by
/// configuration alone.
#[derive(Clone)]
pub enum SearchEngineBackend {
    Elasticsearch(ElasticSearchEngine<ElasticSearchParameterResolver<PGConnection>>),
    Postgres(PgSearchEngine<PgSearchParameterResolver<PGConnection>>),
}

impl SearchEngineBackend {
    /// Whether the backend is reachable, for a caller that waits on it at
    /// startup.
    ///
    /// # Errors
    ///
    /// Returns an error if the backend cannot be reached.
    pub async fn is_connected(&self) -> Result<(), OperationOutcomeError> {
        match self {
            SearchEngineBackend::Elasticsearch(engine) => {
                engine.is_connected().await.map_err(|e| {
                    OperationOutcomeError::fatal(
                        IssueType::exception(),
                        format!("Elasticsearch is not reachable: {e:?}"),
                    )
                })
            }
            SearchEngineBackend::Postgres(engine) => engine.is_connected().await,
        }
    }

    /// What this backend is, for logging that has to say which one it waited
    /// on.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            SearchEngineBackend::Elasticsearch(_) => "Elasticsearch",
            SearchEngineBackend::Postgres(_) => "PostgreSQL search",
        }
    }
}

impl SearchEngine for SearchEngineBackend {
    async fn search(
        &self,
        fhir_version: &haste_repository::types::SupportedFHIRVersions,
        tenant: &haste_jwt::TenantId,
        project: &haste_jwt::ProjectId,
        search_request: &haste_fhir_client::request::SearchRequest,
        options: Option<SearchOptions>,
    ) -> Result<SearchReturn, OperationOutcomeError> {
        match self {
            SearchEngineBackend::Elasticsearch(e) => {
                e.search(fhir_version, tenant, project, search_request, options)
                    .await
            }
            SearchEngineBackend::Postgres(e) => {
                e.search(fhir_version, tenant, project, search_request, options)
                    .await
            }
        }
    }

    async fn index(
        &self,
        fhir_version: haste_repository::types::SupportedFHIRVersions,
        resource: Vec<IndexResource>,
    ) -> Result<IndexOutcome, OperationOutcomeError> {
        match self {
            SearchEngineBackend::Elasticsearch(e) => e.index(fhir_version, resource).await,
            SearchEngineBackend::Postgres(e) => e.index(fhir_version, resource).await,
        }
    }

    async fn migrate(
        &self,
        fhir_version: &haste_repository::types::SupportedFHIRVersions,
    ) -> Result<(), OperationOutcomeError> {
        match self {
            SearchEngineBackend::Elasticsearch(e) => e.migrate(fhir_version).await,
            SearchEngineBackend::Postgres(e) => e.migrate(fhir_version).await,
        }
    }
}

/// Builds the engine `config` describes, against `repo`.
///
/// `migrations` says whether this process is allowed to change the index's
/// shape. The indexing worker only ever writes documents, so it passes false
/// and never prunes a mapping out from under the server.
///
/// # Errors
///
/// Returns an error if the backend cannot be reached or configured.
pub async fn create_search_engine(
    config: &SearchConfig,
    repo: Arc<PGConnection>,
    migrations: bool,
) -> Result<Arc<SearchEngineBackend>, OperationOutcomeError> {
    match config {
        SearchConfig::Elasticsearch(es_config) => {
            let client = create_es_client(
                &es_config.url,
                es_config.username.clone(),
                es_config.password.clone(),
            )?;

            let engine = ElasticSearchEngine::new(
                Arc::new(ElasticSearchParameterResolver::new(client.clone(), repo)),
                Arc::new(FPEngine::new()),
                client,
                migrations && es_config.prune_removed_search_parameters,
            );

            Ok(Arc::new(SearchEngineBackend::Elasticsearch(engine)))
        }
        SearchConfig::Postgres(pg_config) => {
            let pool =
                create_pg_search_pool(&pg_config.database_url, pg_config.max_connections).await?;

            let engine = PgSearchEngine::new(
                Arc::new(PgSearchParameterResolver::new(pool.clone(), repo)),
                Arc::new(FPEngine::new()),
                pool,
            );

            Ok(Arc::new(SearchEngineBackend::Postgres(engine)))
        }
    }
}
