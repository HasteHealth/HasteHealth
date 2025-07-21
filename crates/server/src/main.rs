#![allow(unused)]
use fhir_client::axum::FHIRRequestExtractor;
use fhir_model::r4::types::{
    Address, Extension as FPExtension, ExtensionValueTypeChoice, FHIRInteger, FHIRString,
    HumanName, Identifier, Patient, ResourceType,
};
use fhir_serialization_json::{
    FHIRJSONDeserializer, FHIRJSONSerializer, derive::FHIRJSONSerialize,
};
use reflect::MetaValue;
use serde::{Deserialize, Serialize};
use std::{env::VarError, sync::Arc, time::Instant};
use thiserror::Error;
use tracing::info;

use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    http::Method,
    response::{Html, IntoResponse},
    routing::{get, post},
};
use axum_extra::routing::{
    // for `Router::typed_*`
    TypedPath,
};
use fhirpath::FPEngine;
use maud::html;
use rand::{distr::Alphanumeric, prelude::*};
use sqlx::Pool;
use sqlx_postgres::{PgPoolOptions, Postgres};
use tower_http::services::ServeDir;
use tower_sessions::SessionManagerLayer;
use tower_sessions_sqlx_store::PostgresStore;

use crate::{
    pg::get_pool,
    repository::{FHIRRepository, ProjectId, TenantId},
};

mod fhir_http;
mod oidc;
mod pg;
mod repository;

#[derive(FHIRJSONSerialize)]
#[fhir_serialize_type = "typechoice"]
enum TypeChoiceEnum {
    String(FHIRString),
    Integer(FHIRInteger),
}

#[derive(Error, Debug)]
enum ServerErrors {
    #[error("Session error")]
    Session(#[from] tower_sessions::session::Error),
    #[error("Database error")]
    Database(#[from] sqlx::Error),
    #[error("Failed to load .env file")]
    DotEnv(#[from] dotenvy::Error),
    #[error("Environment variable not set {0}")]
    EnvironmentVariable(#[from] VarError),
    #[error("FHIRPath error")]
    FHIRPath(#[from] fhirpath::FHIRPathError),
    #[error("Failed to deserialize resource")]
    Deserialize(#[from] serde_json::Error),
    #[error("Failed to render template.")]
    TemplateRender,
}

impl IntoResponse for ServerErrors {
    fn into_response(self) -> axum::response::Response {
        match self {
            ServerErrors::Session(e) => (
                axum::http::StatusCode::BAD_REQUEST,
                format!("Session error: {}", e),
            )
                .into_response(),
            ServerErrors::EnvironmentVariable(var) => (
                axum::http::StatusCode::BAD_REQUEST,
                format!("Environment variable not set: {}", var),
            )
                .into_response(),
            ServerErrors::DotEnv(e) => (
                axum::http::StatusCode::BAD_REQUEST,
                format!("Failed to load .env file: {}", e),
            )
                .into_response(),
            ServerErrors::Database(e) => (
                axum::http::StatusCode::BAD_REQUEST,
                format!("Database error: {}", e),
            )
                .into_response(),
            ServerErrors::FHIRPath(e) => (
                axum::http::StatusCode::BAD_REQUEST,
                format!("FP Error {}", e),
            )
                .into_response(),
            ServerErrors::Deserialize(e) => (
                axum::http::StatusCode::BAD_REQUEST,
                format!("Failed to deserialize resource: {}", e),
            )
                .into_response(),
            ServerErrors::TemplateRender => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to render template.".to_string(),
            )
                .into_response(),
        }
    }
}

struct AppState<Store: repository::FHIRRepository> {
    fhir_store: Store,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, serde::Deserialize, serde::Serialize)]
#[sqlx(type_name = "fhir_version", rename_all = "lowercase")]
pub enum FHIRVersion {
    R4,
    R4B,
    R5,
}

struct FHIRHandlerPath {
    tenant: TenantId,
    project: ProjectId,
    fhir_version: FHIRVersion,
    fhir_location: String,
}

async fn fhir_handler(
    Path(fhir_path): Path<FHIRHandlerPath>,
    State(state): State<Arc<AppState<repository::postgres::PostgresSQL>>>,
    request: FHIRRequestExtractor,
) -> Result<Json<serde_json::Value>, ServerErrors> {
    let start = Instant::now();
    info!(
        "Received request for tenant: {}, project: {}, version: {}, location: {}",
        tenant, project, fhir_version, fhir_location
    );

    let response = state
        .fhir_store
        .handle_request(tenant, project, fhir_version, fhir_location, request)
        .await?;

    info!("Request processed in {:?}", start.elapsed());
    Ok(Json(response))
}

#[tokio::main]
async fn main() -> Result<(), ServerErrors> {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    dotenvy::dotenv()?;

    let database_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&std::env::var("DATABASE_URL")?)
        .await?;

    let pool = get_pool().await;
    let store = repository::postgres::PostgresSQL::new(pool.clone()).await?;
    let session_store = PostgresStore::new(pool.clone());

    session_store.migrate().await?;

    let session_layer = SessionManagerLayer::new(session_store).with_secure(true);
    let shared_state = Arc::new(AppState { fhir_store: store });

    let app = Router::new()
        .route("/test/fhirpath", get(test_bed_get))
        .route(
            "/{tenant}/api/v1/{project}/fhir/{:fhir_version}/{:fhir_location}",
            all(fhir_handler),
        )
        .route("/fhirpath", get(fp_handler))
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
