use crate::config::{SearchConfig, SecretProviderConfig, ServerConfig};
use crate::fhir_client::{FHIRServerClient, ServerClientConfig};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use haste_fhir_search::elastic_search::SearchConfigError;
use haste_fhir_search::elastic_search::search_parameter_resolver::ElasticSearchParameterResolver;
use haste_fhir_search::{
    SearchEngine,
    elastic_search::{ElasticSearchEngine, create_es_client},
};
use haste_fhir_terminology::{FHIRTerminology, client::FHIRCanonicalTerminology};
use haste_fhirpath::FPEngine;
use haste_repository::{Repository, pg::PGConnection};
use sqlx::{Pool, Postgres};
use sqlx_postgres::PgPoolOptions;
use std::{env::VarError, sync::Arc};
use tokio::sync::OnceCell;
use tracing::info;

// Singleton for the database connection pool in postgres.
static POOL: OnceCell<Pool<Postgres>> = OnceCell::const_new();
pub async fn get_pool(config: &ServerConfig) -> &'static Pool<Postgres> {
    match &config.repo {
        crate::config::RepoConfig::Postgres(pg_config) => {
            POOL.get_or_init(async || {
                info!("Connecting to postgres database");
                let connection = PgPoolOptions::new()
                    .max_connections(pg_config.max_connections)
                    .connect(&pg_config.database_url)
                    .await
                    .expect("Failed to create database connection pool");
                connection
            })
            .await
        }
    }
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
    FHIRPath(#[from] haste_fhirpath::FHIRPathError),
    #[error(code = "invalid", diagnostic = "Failed to deserialize resource")]
    Deserialize(#[from] serde_json::Error),
    #[error(code = "invalid", diagnostic = "Internal server error")]
    InternalServerError,
}

pub struct ServerState<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
> {
    pub terminology: Arc<Terminology>,
    pub search: Arc<Search>,
    pub repo: Arc<Repo>,
    pub rate_limit: Arc<dyn haste_rate_limit::RateLimit>,
    pub fhir_client: Arc<FHIRServerClient<Repo, Search, Terminology>>,
    pub secret_provider: Arc<dyn haste_encryption::SecretsProvider + Send + Sync>,
    pub config: Arc<crate::config::ServerConfig>,
}

impl<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
> ServerState<Repo, Search, Terminology>
{
    pub async fn transaction(&self) -> Result<Self, OperationOutcomeError> {
        self.repo.transaction(true).await.map(|tx_repo| {
            let tx_repo = Arc::new(tx_repo);
            ServerState {
                terminology: self.terminology.clone(),
                search: self.search.clone(),
                repo: tx_repo.clone(),
                rate_limit: self.rate_limit.clone(),
                secret_provider: self.secret_provider.clone(),
                fhir_client: Arc::new(FHIRServerClient::new(ServerClientConfig::new(
                    tx_repo,
                    self.search.clone(),
                    self.terminology.clone(),
                    self.config.clone(),
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
                    IssueType::EXCEPTION,
                    "Failed to unwrap transaction client".to_string(),
                )
            })?
            .commit()
            .await?;

        Ok(())
    }
}

fn create_search_engine<Repo: Repository + Send + Sync + 'static>(
    config: &crate::config::ServerConfig,
    parameter_resolver: Arc<Repo>,
) -> Result<Arc<ElasticSearchEngine<ElasticSearchParameterResolver<Repo>>>, SearchConfigError> {
    match &config.search {
        SearchConfig::Elasticsearch(elasticsearch_config) => {
            let es_client = create_es_client(
                &elasticsearch_config.url,
                elasticsearch_config.username.clone(),
                elasticsearch_config.password.clone(),
            )?;
            let k = Arc::new(haste_fhir_search::elastic_search::ElasticSearchEngine::new(
                Arc::new(ElasticSearchParameterResolver::new(
                    es_client.clone(),
                    parameter_resolver,
                )),
                Arc::new(FPEngine::new()),
                es_client,
            ));

            Ok(k)
        }
    }
}

pub async fn create_services(
    config: Arc<crate::config::ServerConfig>,
) -> Result<
    Arc<
        ServerState<
            PGConnection,
            ElasticSearchEngine<ElasticSearchParameterResolver<PGConnection>>,
            FHIRCanonicalTerminology,
        >,
    >,
    OperationOutcomeError,
> {
    let pool = Arc::new(PGConnection::pool(get_pool(config.as_ref()).await.clone()));

    let terminology = Arc::new(FHIRCanonicalTerminology::new());

    let search_engine = create_search_engine(config.as_ref(), pool.clone())?;

    let fhir_client = Arc::new(FHIRServerClient::new(
        ServerClientConfig::new(
            pool.clone(),
            search_engine.clone(),
            terminology.clone(),
            config.clone(),
        )
        .with_mutate_artifacts(config.allow_artifact_mutations)
        .with_audit_repo(if config.monitoring.audit_enabled {
            Some(pool.clone())
        } else {
            None
        }),
    ));

    let shared_state = Arc::new(ServerState {
        config: config.clone(),
        rate_limit: pool.clone(),
        repo: pool,
        terminology: terminology,
        search: search_engine,
        fhir_client,
        secret_provider: match &config.security.encryption {
            SecretProviderConfig::Environment { prefix } => haste_encryption::get_secrets_provider(
                haste_encryption::SecretsProviderKind::Environment {
                    prefix: prefix.clone(),
                },
            ),
            _ => {
                return Err(OperationOutcomeError::fatal(
                    IssueType::EXCEPTION,
                    "Only environment encryption is supported for now.".to_string(),
                ));
            }
        },
    });

    Ok(shared_state)
}
