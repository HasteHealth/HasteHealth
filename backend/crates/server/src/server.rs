use crate::{
    auth_n::{self, certificates::get_certification_provider, middleware::jwt::User},
    config::ServerConfig,
    fhir_client::ServerCTX,
    fhir_http::{HTTPBody, HTTPRequest, http_request_to_fhir_request},
    mcp,
    middleware::{
        errors::{log_operationoutcome_errors, operation_outcome_error_handle},
        security_headers::SecurityHeaderLayer,
    },
    openapi,
    services::{ConfigError, ServerState, create_services, get_pool},
    static_assets::{create_static_server, root_asset_route},
};
use axum::{
    Extension, Router, ServiceExt,
    body::Body,
    extract::{DefaultBodyLimit, OriginalUri, Path, State},
    http::Request,
    http::{HeaderName, HeaderValue, Method, Uri},
    middleware::from_fn,
    response::{IntoResponse, Response},
    routing::{any, get, post},
};
use axum_client_ip::ClientIpSource;
use haste_fhir_client::FHIRClient;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, TenantId};
use haste_repository::{Repository, types::SupportedFHIRVersions, utilities::generate_id};
use sentry::integrations::tower::NewSentryLayer;
use serde::Deserialize;
use std::net::SocketAddr;
use std::{collections::HashMap, sync::Arc};
use tower::{Layer, ServiceBuilder};
use tower_http::{catch_panic::CatchPanicLayer, normalize_path::NormalizePath};
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    normalize_path::NormalizePathLayer,
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};
use tower_sessions::{
    Expiry, SessionManagerLayer,
    cookie::{SameSite, time::Duration},
};
use tower_sessions_sqlx_store::PostgresStore;

const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

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
    user: Arc<User>,
    method: Method,
    uri: Uri,
    path: FHIRHandlerPath,
    state: Arc<ServerState<Repo, Search, Terminology>>,
    body: String,
) -> Result<Response, OperationOutcomeError> {
    let fhir_location = path.fhir_location.unwrap_or_default();

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

        let ctx = ServerCTX::new(
            path.tenant,
            path.project,
            path.fhir_version,
            user.clone(),
            state.fhir_client.clone(),
            state.rate_limit.clone(),
        )
        .with_tracing_id(Some(format!("rest-{}", generate_id(Some(8)))));

        let ctx = Arc::new(ctx);

        let response = state.fhir_client.request(ctx, fhir_request).await?;

        let http_response = response.into_response();
        Ok(http_response)
    }
    .await
}

async fn fhir_root_handler<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    method: Method,
    Extension(user): Extension<Arc<User>>,
    OriginalUri(uri): OriginalUri,
    Path(path): Path<FHIRRootHandlerPath>,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
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
    Extension(user): Extension<Arc<User>>,
    OriginalUri(uri): OriginalUri,
    Path(path): Path<FHIRHandlerPath>,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
    body: String,
) -> Result<Response, OperationOutcomeError> {
    fhir_handler(user, method, uri, path, state, body).await
}

pub async fn server(
    config: Arc<ServerConfig>,
) -> Result<NormalizePath<Router>, OperationOutcomeError> {
    let ip_source = match &config.monitoring.ip_source {
        crate::config::IpSource::ConnectInfo => ClientIpSource::ConnectInfo,
        crate::config::IpSource::CfConnectingIp => ClientIpSource::CfConnectingIp,
        crate::config::IpSource::XRealIp => ClientIpSource::XRealIp,
    };

    get_certification_provider(config.as_ref());

    let pool = get_pool(config.as_ref()).await;
    let session_store = PostgresStore::new(pool.clone());
    session_store.migrate().await.map_err(ConfigError::from)?;

    let max_body_size = config.max_request_body_size;
    let shared_state = create_services(config).await?;

    let fhir_router = Router::new()
        .route("/{fhir_version}", any(fhir_root_handler))
        .route("/{fhir_version}/{*fhir_location}", any(fhir_type_handler));

    let protected_resources_router = Router::new()
        .nest("/fhir", fhir_router)
        .route("/mcp", post(mcp::route::mcp_handler))
        .layer(
            ServiceBuilder::new()
                .layer(axum::middleware::from_fn_with_state(
                    shared_state.clone(),
                    auth_n::middleware::basic_auth::basic_auth_middleware,
                ))
                .layer(axum::middleware::from_fn_with_state(
                    shared_state.clone(),
                    auth_n::middleware::jwt::token_verifcation,
                ))
                .layer(axum::middleware::from_fn(
                    auth_n::middleware::project_access::project_access,
                )),
        );

    let project_router = Router::new().merge(protected_resources_router).nest(
        "/oidc",
        auth_n::oidc::routes::create_router(shared_state.clone()),
    );

    let tenant_router = Router::new()
        .nest("/auth", auth_n::tenant::routes::create_router())
        .nest("/{project}/api/v1", project_router)
        .nest(
            "/mfa",
            auth_n::mfa::routes::create_router(shared_state.clone()),
        )
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
        .nest(
            "/auth",
            auth_n::global::routes::create_router(shared_state.clone()),
        )
        .route("/openapi.json", get(openapi::openapi_document_handler))
        .nest("/w/{tenant}", tenant_router)
        .layer(
            ServiceBuilder::new()
                .layer(CatchPanicLayer::new())
                .layer(ip_source.into_extension())
                .layer(NewSentryLayer::<Request<Body>>::new_from_top())
                .layer(TraceLayer::new_for_http())
                // 4mb by default.
                .layer(DefaultBodyLimit::max(max_body_size))
                .layer(CompressionLayer::new())
                .layer(SecurityHeaderLayer::new())
                .layer(SetResponseHeaderLayer::overriding(
                    HeaderName::from_static("x-api-version"),
                    HeaderValue::from_static(SERVER_VERSION),
                ))
                .layer(
                    SessionManagerLayer::new(session_store)
                        .with_secure(true)
                        .with_same_site(SameSite::None)
                        .with_expiry(Expiry::OnInactivity(Duration::days(3))),
                )
                .layer(
                    CorsLayer::new()
                        .allow_methods(Any)
                        .allow_origin(Any)
                        .allow_headers(Any),
                ),
        )
        .with_state(shared_state)
        .nest(root_asset_route().to_str().unwrap(), create_static_server());

    Ok(NormalizePathLayer::trim_trailing_slash().layer(app))
}

pub async fn serve(config: Arc<ServerConfig>, port: u16) -> Result<(), OperationOutcomeError> {
    let server = server(config).await?;

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    tracing::info!("Server started");
    axum::serve(
        listener,
        <tower_http::normalize_path::NormalizePath<Router> as ServiceExt<
            axum::http::Request<Body>,
        >>::into_make_service_with_connect_info::<SocketAddr>(server),
    )
    .await
    .unwrap();

    Ok(())
}
