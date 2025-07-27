#![allow(unused)]
use crate::{
    fhir_http::request::{HTTPRequest, http_request_to_fhir_request},
    pg::get_pool,
    repository::{FHIRMethod, FHIRRepository, InsertResourceRow, ProjectId, TenantId},
};
use axum::{
    Extension, Router, debug_handler,
    extract::{Path, State},
    http::Method,
    response::{IntoResponse, Response},
    routing::any,
};
use fhir_client::request::FHIRRequest;
use fhir_model::r4::sqlx::FHIRJsonRef;
use fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use fhirpath::FPEngine;
use serde::Deserialize;
use std::{env::VarError, sync::Arc, time::Instant};
use tower_http::services::ServeDir;
use tower_sessions::SessionManagerLayer;
use tower_sessions_sqlx_store::PostgresStore;
use tracing::info;

mod config;
mod fhir_http;
mod oidc;
mod pg;
mod repository;

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
    FHIRPath(#[from] fhirpath::FHIRPathError),
    #[error(code = "invalid", diagnostic = "Failed to deserialize resource")]
    Deserialize(#[from] serde_json::Error),
    #[error(code = "invalid", diagnostic = "Internal server error")]
    InternalServerError,
}

struct AppState<Store: repository::FHIRRepository> {
    fhir_store: Store,
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

#[debug_handler]
async fn fhir_handler(
    method: Method,
    Path(path): Path<FHIRHandlerPath>,
    State(state): State<Arc<AppState<repository::postgres::PostgresSQL>>>,
    body: String,
) -> Result<Response, OperationOutcomeError> {
    let start = Instant::now();
    info!("[{}] '{}'", method, path.fhir_location);

    let http_req = HTTPRequest::new(method, path.fhir_location, body);
    let mut fhir_request = http_request_to_fhir_request(SupportedFHIRVersions::R4, &http_req)?;

    info!("Request processed in {:?}", start.elapsed());

    if let FHIRRequest::Create(create_request) = &mut fhir_request {
        repository::utilities::set_resource_id(&mut create_request.resource)?;
        repository::utilities::set_version_id(&mut create_request.resource)?;
    }

    if let FHIRRequest::Create(create_request) = &fhir_request {
        let response = state
            .fhir_store
            .insert(&InsertResourceRow {
                tenant: path.tenant.to_string(),
                project: path.project.to_string(),
                author_id: "fake_author_id".to_string(),
                fhir_version: path.fhir_version,
                resource: FHIRJsonRef(&create_request.resource),
                deleted: false,
                request_method: "POST".to_string(),
                author_type: "member".to_string(),
                fhir_method: FHIRMethod::try_from(&fhir_request).unwrap(),
            })
            .await?;
        Ok((
            axum::http::StatusCode::CREATED,
            fhir_serialization_json::to_string(&response).unwrap(),
        )
            .into_response())
    } else {
        Ok((axum::http::StatusCode::OK, "Request successful".to_string()).into_response())
    }
}

#[tokio::main]
async fn main() -> Result<(), OperationOutcomeError> {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).unwrap();
    let pool = get_pool().await;
    let store = repository::postgres::PostgresSQL::new(pool.clone());
    let session_store = PostgresStore::new(pool.clone());

    session_store.migrate().await.map_err(ConfigError::from)?;

    let session_layer = SessionManagerLayer::new(session_store).with_secure(true);
    let shared_state = Arc::new(AppState { fhir_store: store });

    let app = Router::new()
        .route(
            "/{tenant}/api/v1/{project}/fhir/{fhir_version}/{fhir_location}",
            any(fhir_handler),
        )
        .nest("/oidc", oidc::create_router())
        .layer(session_layer)
        .with_state(shared_state)
        .layer(Extension(Arc::new(FPEngine::new())))
        .fallback_service(ServeDir::new("public"));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    info!("Server started");
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
