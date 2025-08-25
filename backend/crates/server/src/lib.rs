use crate::{
    auth_n::certificates::{JSONWebKeySet, JWK_SET},
    fhir_client::{FHIRServerClient, ServerCTX},
    fhir_http::{HTTPRequest, http_request_to_fhir_request},
    pg::get_pool,
};
use axum::{
    Json, Router,
    extract::{OriginalUri, Path, State},
    http::{Method, Uri},
    response::{IntoResponse, Response},
    routing::{self, any},
};
use oxidized_config::{Config, get_config};
use oxidized_fhir_client::FHIRClient;
use oxidized_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use oxidized_fhir_search::{SearchEngine, elastic_search::ElasticSearchEngine};
use oxidized_fhirpath::FPEngine;
use oxidized_repository::{
    Repository,
    pg::PGConnection,
    types::{Author, ProjectId, SupportedFHIRVersions, TenantId},
};
use serde::Deserialize;
use std::{env::VarError, sync::Arc, time::Instant};
use tower::{Layer, ServiceBuilder};
use tower_http::{
    cors::{Any, CorsLayer},
    normalize_path::NormalizePathLayer,
};
use tower_http::{normalize_path::NormalizePath, services::ServeDir};
use tower_sessions::{
    Expiry, SessionManagerLayer,
    cookie::{SameSite, time::Duration},
};
use tower_sessions_sqlx_store::PostgresStore;
use tracing::info;

mod auth_n;
mod extract;
mod fhir_client;
mod fhir_http;
mod pg;

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
    pub repo: Repo,
    pub fhir_client: FHIRServerClient<Repo, Search>,
    pub config: Box<dyn Config>,
}

#[derive(Deserialize)]
struct FHIRHandlerPath {
    tenant: TenantId,
    project: ProjectId,
    fhir_version: SupportedFHIRVersions,
    fhir_location: Option<String>,
}

#[derive(Deserialize)]
struct FHIRRootHandlerPath {
    tenant: TenantId,
    project: ProjectId,
    fhir_version: SupportedFHIRVersions,
}

async fn fhir_handler<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
>(
    method: Method,
    uri: Uri,
    path: FHIRHandlerPath,
    state: Arc<AppState<Repo, Search>>,
    body: String,
) -> Result<Response, OperationOutcomeError> {
    let start = Instant::now();
    let fhir_location = path.fhir_location.unwrap_or_default();
    info!("[{}] '{}'", method, fhir_location);

    let http_req = HTTPRequest::new(
        method,
        fhir_location,
        body,
        uri.query().unwrap_or_default().to_string(),
    );

    let fhir_request = http_request_to_fhir_request(SupportedFHIRVersions::R4, &http_req)?;

    let ctx = ServerCTX {
        tenant: path.tenant,
        project: path.project,
        fhir_version: path.fhir_version,
        author: Author {
            id: "anonymous".to_string(),
            kind: "Membership".to_string(),
        },
    };

    let response = state.fhir_client.request(ctx, fhir_request).await?;
    info!("Request processed in {:?}", start.elapsed());

    Ok(response.into_response())
}

async fn fhir_root_handler<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
>(
    method: Method,
    OriginalUri(uri): OriginalUri,
    Path(path): Path<FHIRRootHandlerPath>,
    State(state): State<Arc<AppState<Repo, Search>>>,
    body: String,
) -> Result<Response, OperationOutcomeError> {
    fhir_handler(
        method,
        uri,
        FHIRHandlerPath {
            tenant: path.tenant,
            project: path.project,
            fhir_version: path.fhir_version,
            fhir_location: None,
        },
        state,
        body,
    )
    .await
}

async fn fhir_type_handler<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
>(
    method: Method,
    OriginalUri(uri): OriginalUri,
    Path(path): Path<FHIRHandlerPath>,
    State(state): State<Arc<AppState<Repo, Search>>>,
    body: String,
) -> Result<Response, OperationOutcomeError> {
    fhir_handler(method, uri, path, state, body).await
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
        fhir_client: FHIRServerClient::new(repo, search_engine),
    });

    Ok(shared_state)
}

pub async fn jwks_get() -> Result<Json<&'static JSONWebKeySet>, OperationOutcomeError> {
    Ok(Json(&*JWK_SET))
}

pub async fn server() -> Result<NormalizePath<Router>, OperationOutcomeError> {
    let config = get_config("environment".into());
    auth_n::certificates::create_certifications(&config).unwrap();
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let pool = get_pool(config.as_ref()).await;
    let session_store = PostgresStore::new(pool.clone());
    session_store.migrate().await.map_err(ConfigError::from)?;

    let shared_state = create_services(config).await?;

    let project_router = Router::new()
        .route("/fhir/{fhir_version}", any(fhir_root_handler))
        .route(
            "/fhir/{fhir_version}/{*fhir_location}",
            any(fhir_type_handler),
        )
        .nest("/oidc", auth_n::oidc::routes::create_router());

    let tenant_router = Router::new().nest("/api/v1/{project}", project_router);

    let app = Router::new()
        .route("/certs/jwks", routing::get(jwks_get))
        .nest("/w/{tenant}", tenant_router)
        .layer(
            ServiceBuilder::new()
                .layer(
                    SessionManagerLayer::new(session_store)
                        .with_secure(true)
                        .with_same_site(SameSite::None)
                        .with_expiry(Expiry::OnInactivity(Duration::hours(2))),
                )
                .layer(
                    CorsLayer::new()
                        // allow `GET` and `POST` when accessing the resource
                        .allow_methods(Any)
                        // allow requests from any origin
                        .allow_origin(Any)
                        .allow_headers(Any),
                ),
        )
        .with_state(shared_state)
        .fallback_service(ServeDir::new("public"));

    Ok(NormalizePathLayer::trim_trailing_slash().layer(app))
}
