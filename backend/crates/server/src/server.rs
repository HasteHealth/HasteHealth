use crate::{
    auth_n::{
        self,
        certificates::{JSONWebKeySet, JWK_SET},
        claims::UserTokenClaims,
    },
    fhir_client::ServerCTX,
    fhir_http::{HTTPBody, HTTPRequest, http_request_to_fhir_request},
    middleware::errors::{log_operationoutcome_errors, operation_outcome_error_handle},
    services::{AppState, ConfigError, create_services, get_pool},
};
use axum::{
    Extension, Json, Router,
    extract::{OriginalUri, Path, State},
    http::{Method, Uri},
    middleware::from_fn,
    response::{IntoResponse, Response},
    routing::{self, any},
};
use oxidized_config::get_config;
use oxidized_fhir_client::FHIRClient;
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_repository::{
    Repository,
    types::{Author, ProjectId, SupportedFHIRVersions, TenantId},
};
use serde::Deserialize;
use std::{path::PathBuf, sync::Arc, time::Instant};
use tower::{Layer, ServiceBuilder};
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    normalize_path::NormalizePathLayer,
};
use tower_http::{normalize_path::NormalizePath, services::ServeDir};
use tower_sessions::{
    Expiry, SessionManagerLayer,
    cookie::{SameSite, time::Duration},
};
use tower_sessions_sqlx_store::PostgresStore;
use tracing::{Instrument, Level, info, span};

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
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    claims: Arc<UserTokenClaims>,
    method: Method,
    uri: Uri,
    path: FHIRHandlerPath,
    state: Arc<AppState<Repo, Search, Terminology>>,
    body: String,
) -> Result<Response, OperationOutcomeError> {
    let start = Instant::now();
    let fhir_location = path.fhir_location.unwrap_or_default();
    let method_str = method.to_string();
    let span = span!(Level::ERROR, "FHIR-HTTP", method_str, fhir_location);
    async {
        let http_req = HTTPRequest::new(
            method,
            fhir_location,
            HTTPBody::String(body),
            uri.query().unwrap_or_default().to_string(),
        );

        let fhir_request = http_request_to_fhir_request(SupportedFHIRVersions::R4, http_req)?;

        let ctx = Arc::new(ServerCTX {
            tenant: path.tenant,
            project: path.project,
            fhir_version: path.fhir_version,
            author: Author {
                id: claims.sub.clone(),
                kind: claims.resource_type.clone(),
            },
        });

        let response = state.fhir_client.request(ctx, fhir_request).await?;

        info!("Request processed in {:?}", start.elapsed());

        let http_response = response.into_response();
        Ok(http_response)
    }
    .instrument(span)
    .await
}

async fn fhir_root_handler<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    method: Method,
    Extension(user): Extension<Arc<UserTokenClaims>>,
    OriginalUri(uri): OriginalUri,
    Path(path): Path<FHIRRootHandlerPath>,
    State(state): State<Arc<AppState<Repo, Search, Terminology>>>,
    body: String,
) -> Result<Response, OperationOutcomeError> {
    fhir_handler(
        user,
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
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    method: Method,
    Extension(user): Extension<Arc<UserTokenClaims>>,
    OriginalUri(uri): OriginalUri,
    Path(path): Path<FHIRHandlerPath>,
    State(state): State<Arc<AppState<Repo, Search, Terminology>>>,
    body: String,
) -> Result<Response, OperationOutcomeError> {
    fhir_handler(user, method, uri, path, state, body).await
}

async fn jwks_get() -> Result<Json<&'static JSONWebKeySet>, OperationOutcomeError> {
    Ok(Json(&*JWK_SET))
}

static SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

fn root_asset_route() -> PathBuf {
    ["/assets", SERVER_VERSION].iter().collect()
}

pub fn asset_route(asset: &str) -> String {
    let path = root_asset_route();
    path.join(asset).to_str().unwrap().to_string()
}

pub async fn server() -> Result<NormalizePath<Router>, OperationOutcomeError> {
    let config = get_config("environment".into());
    auth_n::certificates::create_certifications(&*config).unwrap();
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let pool = get_pool(config.as_ref()).await;
    let session_store = PostgresStore::new(pool.clone());
    session_store.migrate().await.map_err(ConfigError::from)?;

    let shared_state = create_services(config).await?;

    let fhir_router = Router::new()
        .route("/{fhir_version}", any(fhir_root_handler))
        .route("/{fhir_version}/{*fhir_location}", any(fhir_type_handler))
        .layer(
            ServiceBuilder::new()
                .layer(axum::middleware::from_fn(
                    auth_n::middleware::jwt::token_verifcation,
                ))
                .layer(axum::middleware::from_fn(
                    auth_n::middleware::project_access::project_access,
                )),
        );

    let project_router = Router::new().nest("/fhir", fhir_router).nest(
        "/oidc",
        auth_n::oidc::routes::create_router(shared_state.clone()),
    );

    let tenant_router = Router::new().nest("/api/v1/{project}", project_router);

    let assets_router = Router::new()
        .fallback_service(ServeDir::new("public").append_index_html_on_directories(true));

    let app = Router::new()
        .route("/certs/jwks", routing::get(jwks_get))
        .nest("/w/{tenant}", tenant_router)
        .layer(
            ServiceBuilder::new()
                .layer(CompressionLayer::new())
                .layer(
                    SessionManagerLayer::new(session_store)
                        .with_secure(true)
                        .with_same_site(SameSite::None)
                        .with_expiry(Expiry::OnInactivity(Duration::days(3))),
                )
                .layer(
                    CorsLayer::new()
                        // allow `GET` and `POST` when accessing the resource
                        .allow_methods(Any)
                        // allow requests from any origin
                        .allow_origin(Any)
                        .allow_headers(Any),
                )
                .layer(from_fn(operation_outcome_error_handle))
                .layer(from_fn(log_operationoutcome_errors)),
        )
        .with_state(shared_state)
        .nest(root_asset_route().to_str().unwrap(), assets_router);

    Ok(NormalizePathLayer::trim_trailing_slash().layer(app))
}
