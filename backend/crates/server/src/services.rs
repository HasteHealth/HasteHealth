use crate::fhir_client::FHIRServerClient;
use oxidized_config::Config;
use oxidized_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use oxidized_fhir_search::{SearchEngine, elastic_search::ElasticSearchEngine};
use oxidized_fhirpath::FPEngine;
use oxidized_repository::{Repository, pg::PGConnection};
use sqlx::{Pool, Postgres};
use sqlx_postgres::PgPoolOptions;
use std::{env::VarError, sync::Arc};
use tokio::sync::OnceCell;
use tracing::info;

// Singleton for the database connection pool in postgres.
static POOL: OnceCell<Pool<Postgres>> = OnceCell::const_new();
pub async fn get_pool(config: &dyn Config) -> &'static Pool<Postgres> {
    POOL.get_or_init(async || {
        let database_url = config
            .get("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        info!("Connecting to postgres database");
        let connection = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to create database connection pool");
        connection
    })
    .await
}

#[derive(OperationOutcomeError, Debug)]
pub enum ConfigError {
    #[error(code = "invalid", diagnostic = "Invalid environment!")]
    DotEnv(#[from] dotenvy::Error),
    #[error(code = "invalid", diagnostic = "Invalid session!")]
    Session(#[from] tower_sessions::session::Error),
    #[error(code = "invalid", diagnostic = "Database error")]
    Database(#[from] sqlx::Error),
    #[error(code = "invalid", diagnostic = "Environment variable not set {arg0}")]
    EnvironmentVariable(#[from] VarError),
    #[error(code = "invalid", diagnostic = "Failed to render template.")]
    TemplateRender,
}

#[derive(OperationOutcomeError, Debug)]
pub enum CustomOpError {
    #[error(code = "invalid", diagnostic = "FHIRPath error")]
    FHIRPath(#[from] oxidized_fhirpath::FHIRPathError),
    #[error(code = "invalid", diagnostic = "Failed to deserialize resource")]
    Deserialize(#[from] serde_json::Error),
    #[error(code = "invalid", diagnostic = "Internal server error")]
    InternalServerError,
}

pub struct AppState<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
> {
    pub search: Search,
    pub repo: Repo,
    pub fhir_client: Arc<FHIRServerClient<Repo, Search>>,
    pub config: Box<dyn Config>,
}

pub async fn create_services(
    config: Box<dyn Config>,
) -> Result<Arc<AppState<PGConnection, ElasticSearchEngine>>, OperationOutcomeError> {
    let pool = get_pool(config.as_ref()).await;
    let search_engine = oxidized_fhir_search::elastic_search::ElasticSearchEngine::new(
        Arc::new(FPEngine::new()),
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
    .expect("Failed to create Elasticsearch client");

    let repo = PGConnection::PgPool(pool.clone());

    let shared_state = Arc::new(AppState {
        config,
        repo: repo.clone(),
        search: search_engine.clone(),
        fhir_client: Arc::new(FHIRServerClient::new(
            Arc::new(repo),
            Arc::new(search_engine),
        )),
    });

    Ok(shared_state)
}
