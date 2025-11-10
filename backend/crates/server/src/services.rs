use crate::{
    ServerEnvironmentVariables,
    fhir_client::{FHIRServerClient, ServerClientConfig},
};
use oxidized_config::Config;
use oxidized_fhir_model::r4::generated::terminology::IssueType;
use oxidized_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use oxidized_fhir_search::{SearchEngine, elastic_search::ElasticSearchEngine};
use oxidized_fhir_terminology::{
    FHIRTerminology,
    client::FHIRCanonicalTerminology,
    resolvers::{self, remote::LRUCanonicalRemoteResolver},
};
use oxidized_fhirpath::FPEngine;
use oxidized_repository::{Repository, pg::PGConnection};
use sqlx::{Pool, Postgres};
use sqlx_postgres::PgPoolOptions;
use std::{env::VarError, sync::Arc};
use tokio::sync::OnceCell;
use tracing::info;

// Singleton for the database connection pool in postgres.
static POOL: OnceCell<Pool<Postgres>> = OnceCell::const_new();
pub async fn get_pool(config: &dyn Config<ServerEnvironmentVariables>) -> &'static Pool<Postgres> {
    POOL.get_or_init(async || {
        let database_url = config
            .get(ServerEnvironmentVariables::DataBaseURL)
            .expect(&format!(
                "'{}' must be set",
                String::from(ServerEnvironmentVariables::DataBaseURL)
            ));
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
    Terminology: FHIRTerminology + Send + Sync + 'static,
> {
    pub terminology: Arc<Terminology>,
    pub search: Arc<Search>,
    pub repo: Arc<Repo>,
    pub fhir_client: Arc<FHIRServerClient<Repo, Search, Terminology>>,
    pub config: Arc<dyn Config<ServerEnvironmentVariables>>,
}

impl<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
> AppState<Repo, Search, Terminology>
{
    pub async fn transaction(&self) -> Result<Self, OperationOutcomeError> {
        self.repo.transaction(None, true).await.map(|tx_repo| {
            let tx_repo = Arc::new(tx_repo);
            AppState {
                terminology: self.terminology.clone(),
                search: self.search.clone(),
                repo: tx_repo.clone(),
                fhir_client: Arc::new(FHIRServerClient::new(ServerClientConfig::new(
                    tx_repo,
                    self.search.clone(),
                    self.terminology.clone(),
                ))),
                config: self.config.clone(),
            }
        })
    }
    pub async fn commit(self) -> Result<(), OperationOutcomeError> {
        let repo = self.repo.clone();
        drop(self);

        Arc::try_unwrap(repo)
            .map_err(|_e| {
                OperationOutcomeError::fatal(
                    IssueType::Exception(None),
                    "Failed to unwrap transaction client".to_string(),
                )
            })?
            .commit()
            .await?;

        Ok(())
    }
}

pub async fn create_services(
    config: Arc<dyn Config<ServerEnvironmentVariables>>,
) -> Result<
    Arc<
        AppState<
            PGConnection,
            ElasticSearchEngine,
            FHIRCanonicalTerminology<LRUCanonicalRemoteResolver<PGConnection, ElasticSearchEngine>>,
        >,
    >,
    OperationOutcomeError,
> {
    let pool = get_pool(config.as_ref()).await;
    let search_engine = Arc::new(
        oxidized_fhir_search::elastic_search::ElasticSearchEngine::new(
            Arc::new(FPEngine::new()),
            &config
                .get(ServerEnvironmentVariables::ElasticSearchURL)
                .expect(&format!(
                    "'{}' variable not set",
                    String::from(ServerEnvironmentVariables::ElasticSearchURL)
                )),
            config
                .get(ServerEnvironmentVariables::ElasticSearchUsername)
                .expect(&format!(
                    "'{}' variable not set",
                    String::from(ServerEnvironmentVariables::ElasticSearchUsername)
                )),
            config
                .get(ServerEnvironmentVariables::ElasticSearchPassword)
                .expect(&format!(
                    "'{}' variable not set",
                    String::from(ServerEnvironmentVariables::ElasticSearchPassword)
                )),
        )
        .expect("Failed to create Elasticsearch client"),
    );

    let repo = Arc::new(PGConnection::pool(pool.clone()));

    let terminology = Arc::new(FHIRCanonicalTerminology::new(
        resolvers::remote::LRUCanonicalRemoteResolver::new(repo.clone(), search_engine.clone()),
    ));

    let can_mutate: String = config
        .get(ServerEnvironmentVariables::AllowArtifactMutations)
        .unwrap_or("false".into());

    let fhir_client = Arc::new(FHIRServerClient::new(if can_mutate == "true" {
        ServerClientConfig::allow_mutate_artifacts(
            repo.clone(),
            search_engine.clone(),
            terminology.clone(),
        )
    } else {
        ServerClientConfig::new(repo.clone(), search_engine.clone(), terminology.clone())
    }));

    let shared_state = Arc::new(AppState {
        config,
        repo: repo,
        terminology: terminology,
        search: search_engine,
        fhir_client,
    });

    Ok(shared_state)
}
