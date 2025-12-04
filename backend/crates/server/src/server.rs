use crate::{
    auth_n,
    fhir_client::ServerCTX,
    fhir_http::{HTTPBody, HTTPRequest, http_request_to_fhir_request},
    middleware::errors::{log_operationoutcome_errors, operation_outcome_error_handle},
    services::{AppState, ConfigError, create_services, get_pool},
    static_assets::{create_static_server, root_asset_route},
};
use axum::{
    Extension, Router, ServiceExt,
    body::Body,
    extract::{DefaultBodyLimit, OriginalUri, Path, State},
    http::{Method, Uri},
    middleware::from_fn,
    response::{IntoResponse, Response},
    routing::{any, get},
};
use haste_config::get_config;
use haste_fhir_client::FHIRClient;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, TenantId, claims::UserTokenClaims};
use haste_repository::{Repository, types::SupportedFHIRVersions};
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc, time::Instant};
use tower::{Layer, ServiceBuilder};
use tower_http::normalize_path::NormalizePath;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    normalize_path::NormalizePathLayer,
};
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
            uri.query()
                .map(|q| {
                    url::form_urlencoded::parse(q.as_bytes())
                        .into_owned()
                        .collect()
                })
                .unwrap_or_else(HashMap::new),
        );

        let fhir_request = http_request_to_fhir_request(SupportedFHIRVersions::R4, http_req)?;

        let ctx = Arc::new(ServerCTX::new(
            path.tenant,
            path.project,
            path.fhir_version,
            claims.clone(),
            state.fhir_client.clone(),
        ));

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

pub async fn server() -> Result<NormalizePath<Router>, OperationOutcomeError> {
    let config = get_config("environment".into());
    auth_n::certificates::create_certifications(&*config).unwrap();
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let pool = get_pool(config.as_ref()).await;
    let session_store = PostgresStore::new(pool.clone());
    session_store.migrate().await.map_err(ConfigError::from)?;

    let max_body_size = config
        .get(crate::ServerEnvironmentVariables::MaxRequestBodySize)
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(4 * 1024 * 1024);
    let shared_state = create_services(config).await?;

    let fhir_router = Router::new()
        .route("/{fhir_version}", any(fhir_root_handler))
        .route("/{fhir_version}/{*fhir_location}", any(fhir_type_handler))
        .layer(
            ServiceBuilder::new()
                .layer(axum::middleware::from_fn_with_state(
                    shared_state.clone(),
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

    let tenant_router = Router::new()
        .nest("/{project}/api/v1", project_router)
        .layer(
            // Relies on tenant for html so moving operation outcome error handling to here.
            ServiceBuilder::new()
                .layer(from_fn(operation_outcome_error_handle))
                .layer(from_fn(log_operationoutcome_errors)),
        );

    let discovery_2_0_document_router = Router::new()
        .route(
            "/openid-configuration/w/{tenant}/{project}/{*resource}",
            get(auth_n::oidc::routes::discovery::openid_configuration),
        )
        .route(
            "/openid-configuration/w/{tenant}/{project}",
            get(auth_n::oidc::routes::discovery::openid_configuration),
        )
        .route(
            "/oauth-protected-resource/w/{tenant}/{project}/{*resource}",
            get(auth_n::oidc::routes::discovery::oauth_protected_resource),
        );

    let app = Router::new()
        .nest("/.well-known", discovery_2_0_document_router)
        .nest("/w/{tenant}", tenant_router)
        .layer(
            ServiceBuilder::new()
                // 4mb by default.
                .layer(DefaultBodyLimit::max(max_body_size))
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
                ),
        )
        .with_state(shared_state)
        .nest(root_asset_route().to_str().unwrap(), create_static_server());

    Ok(NormalizePathLayer::trim_trailing_slash().layer(app))
}

pub async fn serve(port: u16) -> Result<(), OperationOutcomeError> {
    let server = server().await?;

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    tracing::info!("Server started");
    axum::serve(
        listener,
        <tower_http::normalize_path::NormalizePath<Router> as ServiceExt<
            axum::http::Request<Body>,
        >>::into_make_service(server),
    )
    .await
    .unwrap();

    Ok(())
}
