#![allow(unused)]
use crate::{
    fhir_http::request::{FHIRRequestParsingError, HTTPRequest, http_request_to_fhir_request},
    pg::get_pool,
    repository::{FHIRMethod, FHIRRepository, InsertResourceRow, ProjectId, TenantId},
};
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
use fhir_client::request::FHIRRequest;
use fhir_model::r4::{
    sqlx::{FHIRJson, FHIRJsonRef},
    types::{
        Address, Extension as FPExtension, ExtensionValueTypeChoice, FHIRId, FHIRInteger,
        FHIRString, HumanName, Identifier, Meta, Patient, Resource, ResourceType,
    },
};
use fhir_operation_error::derive::OperationOutcomeError;
use fhir_serialization_json::{
    FHIRJSONDeserializer, FHIRJSONSerializer, derive::FHIRJSONSerialize,
};
use fhirpath::FPEngine;
use maud::html;
use rand::{distr::Alphanumeric, prelude::*};
use reflect::MetaValue;
use serde::{Deserialize, Serialize};
use sqlx::Pool;
use sqlx_postgres::{PgPoolOptions, Postgres};
use std::{env::VarError, io::BufWriter, sync::Arc, time::Instant};
use thiserror::Error;
use tower_http::services::ServeDir;
use tower_sessions::SessionManagerLayer;
use tower_sessions_sqlx_store::PostgresStore;
use tracing::info;

mod fhir_http;
mod oidc;
mod pg;
mod repository;

#[derive(OperationOutcomeError)]
pub enum CustomOpError {
    #[information(code = "info", diagnostic = "Informational message")]
    #[fatal(code = "invalid", diagnostic = "Not Found")]
    NotFound,
    #[error(code = "not-found", diagnostic = "Resource not found")]
    InvalidInput,
}

// [A-Za-z0-9\-\.]{1,64} See https://hl7.org/fhir/r4/datatypes.html#id
// Can't use _ for compliance.
fn generate_id() -> String {
    nanoid::nanoid!(
        26,
        &[
            '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f', 'g',
            'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x',
            'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O',
            'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '-'
        ]
    )
    .to_string()
}

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
    #[error("Failed to parse FHIR request.")]
    FHIRRequestParsingError(#[from] FHIRRequestParsingError),
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
            ServerErrors::FHIRRequestParsingError(fhirrequest_parsing_error) => (
                axum::http::StatusCode::BAD_REQUEST,
                "Failed to parse FHIR Request.",
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

fn set_resource_id(resource: &mut Resource) -> Result<(), crate::ServerErrors> {
    let mut id: &mut dyn std::any::Any = resource
        .get_field_mut("id")
        .ok_or(ServerErrors::InternalServerError)?;
    let id: &mut Option<String> = id
        .downcast_mut::<Option<String>>()
        .ok_or(ServerErrors::InternalServerError)?;
    *id = Some(generate_id());
    Ok(())
}

fn set_version_id(resource: &mut Resource) -> Result<(), crate::ServerErrors> {
    let mut meta: &mut dyn std::any::Any = resource
        .get_field_mut("meta")
        .ok_or(ServerErrors::InternalServerError)?;
    let meta: &mut Option<Box<Meta>> = meta
        .downcast_mut::<Option<Box<Meta>>>()
        .ok_or(ServerErrors::InternalServerError)?;

    if meta.is_none() {
        *meta = Some(Box::new(Meta::default()))
    }
    meta.as_mut().map(|meta| {
        meta.versionId = Some(Box::new(FHIRId {
            id: None,
            extension: None,
            value: Some(generate_id()),
        }));
    });

    Ok(())
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
    let mut fhir_request = http_request_to_fhir_request(SupportedFHIRVersions::R4, &http_req)?;

    info!("Request processed in {:?}", start.elapsed());

    if let FHIRRequest::Create(create_request) = &mut fhir_request {
        set_resource_id(&mut create_request.resource)?;
        set_version_id(&mut create_request.resource)?;
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
            fhir_serialization_json::to_string(&create_request.resource).unwrap(),
        )
            .into_response())
    } else {
        Ok((axum::http::StatusCode::OK, "Request successful".to_string()).into_response())
    }
}

struct Z(String, String);

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

    let Z(a0, a1) = Z("asdf".to_string(), "qwer".to_string());
    format!("{a0}, {a1}");

    let op: OperationError = CustomOpError::NotFound.into();
    info!("Operation outcome: {:?}", op.outcome());
    info!("Server started");
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
