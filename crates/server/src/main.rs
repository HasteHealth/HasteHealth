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

use crate::{pg::get_pool, repository::FHIRRepository};

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
#[sqlx(type_name = "fhir_method", rename_all = "lowercase")]
pub enum FHIRMethod {
    Create,
    Delete,
    Patch,
    Update,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, serde::Deserialize, serde::Serialize)]
#[sqlx(type_name = "fhir_version", rename_all = "lowercase")]
pub enum FHIRVersion {
    R4,
    R4B,
    R5,
}

#[derive(Clone, Debug, sqlx::FromRow, Serialize)]
struct ResourceRow {
    #[serde(rename(serialize = "_id"))]
    id: String,
    resource: serde_json::Value,
    fhir_method: FHIRMethod,
}

fn get_random_string(rng: &mut StdRng) -> String {
    rng.sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect()
}

async fn post_resource<FR: FHIRRepository>(
    State(state): State<Arc<AppState<FR>>>,
    // Json(payload): Json<serde_json::Value>,
    FHIRRequestExtractor(fhir_request): FHIRRequestExtractor,
) -> Result<String, ServerErrors> {
    // println!("RECEIVED PAYLOAD: {0:?}", payload);

    // let now = Instant::now();

    // let resource = from_value::<Resource>(payload)?;

    // let elapsed = now.elapsed();

    // println!("Serde Deserialization time: {:.2?}", elapsed);

    // let now = Instant::now();
    // let json = fhir_serialization_json::to_string(&resource);
    // let elapsed = now.elapsed();

    // println!("Serde Serialization time: {:.2?}", elapsed);
    // println!("Received resource: {0:?}", resource);

    Ok("OK".to_string())
}

#[derive(Deserialize)]
struct FPQuerys {
    query: String,
}

async fn fp_handler<FR: FHIRRepository>(
    method: Method,
    State(state): State<Arc<AppState<FR>>>,
    Query(fp): Query<FPQuerys>,
    Extension(fp_engine): Extension<Arc<FPEngine>>,
) -> Result<String, ServerErrors> {
    let mut patient = Patient::default();
    let mut name = HumanName::default();
    let mut identifier = Identifier::default();
    let mut given = FHIRString::default();

    let mut id_extension = FPExtension::default();
    id_extension.url = "http://example.com/fhir/StructureDefinition/patient-id".to_string();

    id_extension.value = Some(ExtensionValueTypeChoice::String(Box::new(FHIRString {
        id: None,
        extension: None,
        value: Some("1234567890".to_string()),
    })));

    println!(
        "{:?}",
        fhir_serialization_json::from_str::<Patient>(
            r#"
    {
        "resourceType": "Patient",
        "id": "123",
        "name": [
            {
                "given": ["Bob"],
                "family": "Smith"
            }
        ],
        "identifier": [
            {
                "id": "asdf",
                "value": "1234567890",
                "_value": {
                    "id": "123",
                    "extension": [
                        {
                            "url": "http://example.com/fhir/StructureDefinition/patient-id",
                            "valueString": "1234567890"
                        }
                    ]
                }
            }
        ]
    }
    "#
        )
    );

    let mut given_extension = FPExtension::default();

    let mut address = Address::default();
    address.extension = Some(vec![Box::new(FPExtension {
        id: None,
        extension: None,
        url: "http://example.com/fhir/StructureDefinition/address-line".to_string(),
        value: Some(ExtensionValueTypeChoice::String(Box::new(FHIRString {
            id: None,
            extension: None,
            value: Some("Myline".to_string()),
        }))),
    })]);

    address.line = Some(vec![
        Box::new(FHIRString {
            id: None,
            extension: None,
            value: Some("Myline".to_string()),
        }),
        Box::new(FHIRString {
            id: None,
            extension: None,
            value: Some("Myline".to_string()),
        }),
        Box::new(FHIRString {
            id: None,
            extension: None,
            value: Some("Myline".to_string()),
        }),
        Box::new(FHIRString {
            id: None,
            extension: None,
            value: Some("Myline".to_string()),
        }),
        Box::new(FHIRString {
            id: Some("Hello world".to_string()),
            extension: None,
            value: None,
        }),
        Box::new(FHIRString {
            id: None,
            extension: None,
            value: Some("Myline".to_string()),
        }),
        Box::new(FHIRString {
            id: None,
            extension: None,
            value: Some("Myline".to_string()),
        }),
    ]);
    given_extension.value = Some(ExtensionValueTypeChoice::Address(Box::new(address)));

    given.value = Some("Bob".to_string());
    given.extension = Some(vec![Box::new(given_extension)]);
    identifier.value = Some(Box::new(FHIRString {
        id: None,
        extension: Some(vec![Box::new(id_extension)]),
        value: Some("1234567890".to_string()),
    }));

    name.given = Some(vec![Box::new(given)]);
    patient.name = Some(vec![Box::new(name)]);
    patient.identifier_ = Some(vec![Box::new(identifier)]);

    let now = Instant::now();

    for i in 0..10000 {
        let result = fp_engine.evaluate(&fp.query, vec![&patient])?;
        let values: Vec<&dyn MetaValue> = result.iter().collect();
    }

    let elapsed = now.elapsed();
    println!("Elapsed time: {:.2?}", elapsed);

    let result = fp_engine.evaluate(&fp.query, vec![&patient])?;
    let values: Vec<&dyn MetaValue> = result.iter().collect();

    let now = Instant::now();
    let serialized = patient.serialize_value().unwrap();
    let elapsed = now.elapsed();
    println!("Custom Serialized time: {:.2?}", elapsed);
    println!("Custom Serialized: {}", serialized);

    Ok(format!("Evaluation: {:?}", values))
}

async fn test_bed_get<FR: FHIRRepository>(
    State(state): State<Arc<AppState<FR>>>,
) -> Result<Html<String>, ServerErrors> {
    let name = "Lyra";
    let markup = html! {
        h1 { "HELLO WORLD!"}
        p { "Hi, " (name) "!" }
    };

    Ok(Html(markup.into_string()))
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
        .route("/{tenant}/api/v1/{project}/fhir/{}", post(post_resource))
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
