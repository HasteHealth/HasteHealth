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
    body::{Body, Bytes},
    extract::{DefaultBodyLimit, OriginalUri, Path, State},
    http::Request,
    http::{HeaderName, HeaderValue, Method},
    middleware::from_fn,
    response::{IntoResponse, Response},
    routing::{any, get, post},
};
use axum_client_ip::ClientIpSource;
use haste_fhir_client::{
    FHIRClient,
    request::{FHIRCapabilitiesResponse, FHIRResponse},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, TenantId};
use haste_repository::{Repository, types::SupportedFHIRVersions, utilities::generate_id};
use sentry::integrations::tower::NewSentryLayer;
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;
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
    /// Not captured by the FHIR root route, which has nothing after the
    /// version, so serde leaves it as `None` there.
    fhir_location: Option<String>,
}

/// The project a route addresses, for handlers that do not act on a FHIR
/// location within it.
#[derive(Deserialize)]
struct ProjectPath {
    tenant: TenantId,
    project: ProjectId,
}

async fn fhir_handler<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    method: Method,
    Extension(user): Extension<Arc<User>>,
    OriginalUri(uri): OriginalUri,
    Path(path): Path<FHIRHandlerPath>,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
    body: Bytes,
) -> Result<Response, OperationOutcomeError> {
    let http_req = HTTPRequest::new(
        method,
        path.fhir_location.unwrap_or_default(),
        HTTPBody::Bytes(body),
        uri.query()
            .map(|q| {
                url::form_urlencoded::parse(q.as_bytes())
                    .into_owned()
                    .collect()
            })
            .unwrap_or_default(),
    );

    let fhir_request = http_request_to_fhir_request(SupportedFHIRVersions::R4, http_req)?;

    let ctx = Arc::new(
        ServerCTX::new(
            path.tenant,
            path.project,
            path.fhir_version,
            user,
            state.fhir_client.clone(),
            state.rate_limit.clone(),
        )
        .with_tracing_id(Some(format!("rest-{}", generate_id(Some(8))))),
    );

    let response = state.fhir_client.request(ctx, fhir_request).await?;

    Ok(response.into_response())
}

async fn public_metadata_handler<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    Path(path): Path<ProjectPath>,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
) -> Result<Response, OperationOutcomeError> {
    let ctx = Arc::new(ServerCTX::system(
        path.tenant,
        path.project,
        state.fhir_client.clone(),
        state.rate_limit.clone(),
    ));

    state
        .fhir_client
        .capabilities(ctx)
        .await
        .map(|capabilities| FHIRResponse::Capabilities(FHIRCapabilitiesResponse { capabilities }))
        .map(|fhir_response| fhir_response.into_response())
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

    let shared_state = create_services(config.clone()).await?;

    // Two routes because a wildcard does not match an empty segment: the first
    // is the FHIR root (system level interactions), the second everything under
    // it. Both are the same handler.
    let fhir_router = Router::new()
        .route("/{fhir_version}", any(fhir_handler))
        .route("/{fhir_version}/{*fhir_location}", any(fhir_handler));

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

    let smart_configuration_router = Router::new()
        // Per spec must be at root of the fhir server which is why /fhir/{fhir_version}/.well-known/smart-configuration is used as the route.
        // Because this is publically available it is not under protected_resources_router and does not require authentication.
        .route(
            "/fhir/{fhir_version}/.well-known/smart-configuration",
            get(auth_n::oidc::routes::discovery::smart_configuration),
        )
        .route_layer(
            ServiceBuilder::new().layer(axum::middleware::from_fn_with_state(
                shared_state.clone(),
                auth_n::oidc::middleware::project_exists,
            )),
        );

    let mut project_router = Router::new()
        .merge(protected_resources_router)
        .merge(smart_configuration_router)
        .nest(
            "/oidc",
            auth_n::oidc::routes::create_router(shared_state.clone()),
        );

    if config.security.publicize_fhir_metadata {
        project_router = project_router.route(
            "/fhir/{fhir_version}/metadata",
            get(public_metadata_handler),
        );
    }

    let tenant_router = Router::new()
        .route("/branding/logo", get(auth_n::tenant::routes::logo))
        .nest("/auth", auth_n::tenant::routes::create_router())
        .nest("/{project}/api/v1", project_router)
        .nest(
            "/mfa",
            auth_n::mfa::routes::create_router(shared_state.clone()),
        )
        .layer(
            // Relies on tenant (and now tenant branding) for html so moving operation outcome error handling to here.
            ServiceBuilder::new()
                .layer(axum::middleware::from_fn_with_state(
                    shared_state.clone(),
                    operation_outcome_error_handle,
                ))
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
        .route(
            "/schemas/fhir/{resource_type}",
            get(openapi::resource_schema_handler),
        )
        .nest("/w/{tenant}", tenant_router)
        .layer(
            ServiceBuilder::new()
                .layer(CatchPanicLayer::new())
                .layer(ip_source.into_extension())
                .layer(NewSentryLayer::<Request<Body>>::new_from_top())
                .layer(TraceLayer::new_for_http())
                // 4mb by default.
                .layer(DefaultBodyLimit::max(config.max_request_body_size))
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
