#![allow(unused)]
use crate::{
    fhir_http::{HTTPRequest, http_request_to_fhir_request},
    pg::get_pool,
    repository::{FHIRRepository, ProjectId, TenantId},
    server_client::{FHIRServerClient, ServerCTX},
};
use axum::{
    Extension, Router,
    extract::{Path, State},
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
    routing::any,
};
use oxidized_config::{Config, get_config};
use oxidized_fhir_client::FHIRClient;
use oxidized_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use oxidized_fhirpath::FPEngine;
use serde::Deserialize;
use std::{env::VarError, sync::Arc, time::Instant};
use tower_http::services::ServeDir;
use tower_sessions::SessionManagerLayer;
use tower_sessions_sqlx_store::PostgresStore;
use tracing::info;

mod fhir_http;
mod oidc;
mod pg;
mod repository;
mod search;
mod server_client;

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

struct AppState<Repo: FHIRRepository + Send + Sync> {
    fhir_client: FHIRServerClient<Repo>,
    config: Box<dyn Config>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, serde::Deserialize, serde::Serialize)]
#[sqlx(type_name = "fhir_version", rename_all = "lowercase")] // only for PostgreSQL to match a type definition
pub enum SupportedFHIRVersions {
    R4,
    R4B,
    R5,
}
impl std::fmt::Display for SupportedFHIRVersions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SupportedFHIRVersions::R4 => write!(f, "R4"),
            SupportedFHIRVersions::R4B => write!(f, "R4B"),
            SupportedFHIRVersions::R5 => write!(f, "R5"),
        }
    }
}

#[derive(Deserialize)]
struct FHIRHandlerPath {
    tenant: TenantId,
    project: ProjectId,
    fhir_version: SupportedFHIRVersions,
    fhir_location: String,
}

async fn fhir_handler<Repo: repository::FHIRRepository + Send + Sync + 'static>(
    method: Method,
    Path(path): Path<FHIRHandlerPath>,
    State(state): State<Arc<AppState<Repo>>>,
    body: String,
) -> Result<Response, OperationOutcomeError> {
    let start = Instant::now();
    info!("[{}] '{}'", method, path.fhir_location);

    let http_req = HTTPRequest::new(method, path.fhir_location, body);
    let fhir_request = http_request_to_fhir_request(SupportedFHIRVersions::R4, &http_req)?;

    let ctx = ServerCTX {
        tenant: path.tenant,
        project: path.project,
        fhir_version: path.fhir_version,
    };

    let response = state.fhir_client.request(ctx, fhir_request).await?;
    info!("Request processed in {:?}", start.elapsed());

    Ok(response.into_response())
}

#[tokio::main]
async fn main() -> Result<(), OperationOutcomeError> {
    let config = get_config("environment".into());
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let pool = get_pool(config.as_ref()).await;
    let session_store = PostgresStore::new(pool.clone());
    session_store.migrate().await.map_err(ConfigError::from)?;

    let shared_state = Arc::new(AppState {
        config,
        fhir_client: FHIRServerClient::new(repository::postgres::FHIRPostgresRepository::new(
            pool.clone(),
        )),
    });

    let app = Router::new()
        .route(
            "/{tenant}/api/v1/{project}/fhir/{fhir_version}/{*fhir_location}",
            any(fhir_handler),
        )
        .nest("/oidc", oidc::create_router())
        .layer(SessionManagerLayer::new(session_store).with_secure(true))
        .with_state(shared_state)
        .layer(Extension(Arc::new(FPEngine::new())))
        .fallback_service(ServeDir::new("public"));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    info!("Server started");
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
