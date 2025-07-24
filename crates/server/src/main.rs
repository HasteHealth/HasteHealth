#![allow(unused)]
use fhir_client::request::FHIRRequest;
use fhir_model::r4::{
    sqlx::FHIRJson,
    types::{
        Address, Extension as FPExtension, ExtensionValueTypeChoice, FHIRInteger, FHIRString,
        HumanName, Identifier, Patient, Resource, ResourceType,
    },
};
use fhir_serialization_json::{
    FHIRJSONDeserializer, FHIRJSONSerializer, derive::FHIRJSONSerialize,
};
use reflect::MetaValue;
use serde::{Deserialize, Serialize};
use std::{env::VarError, io::BufWriter, sync::Arc, time::Instant};
use thiserror::Error;
use tracing::info;

use axum::{
    Extension, Json, Router,
    body::Body,
    debug_handler,
    extract::{Path, Query, Request, State},
    http::Method,
    response::{Html, IntoResponse, Response},
    routing::{any, get, post},
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
    fhir_http::request::{HTTPRequest, http_request_to_fhir_request},
    pg::get_pool,
    repository::{FHIRMethod, FHIRRepository, InsertResourceRow, ProjectId, TenantId},
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
    #[error("Internal server error")]
    InternalServerError,
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
            ServerErrors::InternalServerError => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            )
                .into_response(),
        }
    }
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

#[derive(Debug)]
struct Tester {
    name: String,
    age: u32,
}

#[debug_handler]
async fn fhir_handler(
    method: Method,
    Path(path): Path<FHIRHandlerPath>,
    State(state): State<Arc<AppState<repository::postgres::PostgresSQL>>>,
    body: String,
) -> Result<Response, ServerErrors> {
    let start = Instant::now();
    info!("[{}] '{}'", method, path.fhir_location);

    let http_req = HTTPRequest::new(method, path.fhir_location, body);
    let mut fhir_request =
        http_request_to_fhir_request(SupportedFHIRVersions::R4, &http_req).unwrap();

    info!("Request processed in {:?}", start.elapsed());

    if let FHIRRequest::Create(create_request) = &mut fhir_request {
        let mut id = create_request.resource.get_field("id").unwrap();
        let z = id.as_any().downcast_mut::<Option<String>>().unwrap();

        *z = Some("random_id".to_string());
    }

    if let FHIRRequest::Create(create_request) = &fhir_request {
        let response = state
            .fhir_store
            .insert(&InsertResourceRow {
                tenant: path.tenant.to_string(),
                project: path.project.to_string(),
                author_id: "fake_author_id".to_string(),
                fhir_version: path.fhir_version,
                resource: FHIRJson(create_request.resource.clone()),
                deleted: false,
                request_method: "POST".to_string(),
                author_type: "member".to_string(),
                fhir_method: FHIRMethod::try_from(&fhir_request).unwrap(),
            })
            .await?;

        println!("Resource inserted: {:?}", response);

        Ok((
            axum::http::StatusCode::CREATED,
            fhir_serialization_json::to_string(&create_request.resource).unwrap(),
        )
            .into_response())
    } else {
        Ok((axum::http::StatusCode::OK, "Request successful".to_string()).into_response())
    }
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

    use std::io::Write;
    let mut k = BufWriter::new(Vec::new());

    k.write("hello".as_bytes());

    Ok(())
}
