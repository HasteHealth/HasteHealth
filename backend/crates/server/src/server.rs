use crate::{
    SERVER_VERSION,
    auth_n::{self, certificates::get_certification_provider, middleware::jwt::User},
    config::ServerConfig,
    fhir_client::ServerCTX,
    fhir_http::{HTTPBody, HTTPRequest, http_request_to_fhir_request},
    jobs, mcp,
    middleware::{
        errors::{REQUEST_ID_HEADER, log_operationoutcome_errors, operation_outcome_error_handle},
        security_headers::security_headers,
    },
    openapi,
    services::{ConfigError, ServerState, create_services, get_pool},
    static_assets::{create_static_server, root_asset_route},
};
use axum::{
    Extension, Router, ServiceExt,
    body::{Body, Bytes, to_bytes},
    extract::{DefaultBodyLimit, OriginalUri, Path, State},
    http::Request,
    http::{HeaderMap, HeaderName, HeaderValue, Method, Uri, header},
    middleware::from_fn,
    response::{IntoResponse, Response},
    routing::{any, get, post},
};
use axum_client_ip::ClientIpSource;
use haste_fhir_client::{
    FHIRClient,
    request::{FHIRCapabilitiesResponse, FHIRRequest, FHIRResponse},
};
use haste_fhir_model::r4::generated::terminology::IssueType;
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
use tower_http::{catch_panic::CatchPanicLayer, cors::AllowHeaders, normalize_path::NormalizePath};
use tower_http::{
    compression::{
        CompressionLayer,
        predicate::{DefaultPredicate, NotForContentType, Predicate},
    },
    cors::{Any, CorsLayer},
    normalize_path::NormalizePathLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};
use tower_sessions::{
    Expiry, SessionManagerLayer,
    cookie::{SameSite, time::Duration},
};
use tower_sessions_sqlx_store::PostgresStore;

/// The project a route addresses. The FHIR version is not part of it: it comes
/// from the project, and the URL only optionally repeats it.
#[derive(Deserialize)]
struct ProjectPath {
    tenant: TenantId,
    project: ProjectId,
}

/// Splits an optional leading FHIR version off a path under the FHIR root.
///
/// Both `r4/Patient/123` and `Patient/123` address the same resource. No FHIR
/// resource type is spelled like a version, so a first segment that names one
/// is the version and never a location.
fn split_fhir_version(path: &str) -> (Option<SupportedFHIRVersions>, &str) {
    let path = path.trim_start_matches('/');
    let (head, rest) = path.split_once('/').unwrap_or((path, ""));

    match SupportedFHIRVersions::from_url_segment(head) {
        Some(version) => (Some(version), rest),
        None => (None, path),
    }
}

/// The FHIR version a request works in.
///
/// It belongs to the project, and the token carries it, so the URL can only
/// agree with it. A request that names a different version is asking for
/// something this project does not serve, and is refused rather than quietly
/// answered in the wrong version.
fn request_fhir_version(
    user: &User,
    path: &ProjectPath,
    requested: Option<SupportedFHIRVersions>,
) -> Result<SupportedFHIRVersions, OperationOutcomeError> {
    let project_version = user.claims.fhir_version.clone();

    if let Some(requested) = requested
        && requested != project_version
    {
        return Err(OperationOutcomeError::error(
            IssueType::not_supported(),
            format!(
                "Project '{}' is served as FHIR {}, not {}.",
                path.project.as_ref(),
                project_version,
                requested
            ),
        ));
    }

    Ok(project_version)
}

/// Whether a search's `query` asks for `_pretty=true`. `_format` must name
/// JSON, the only format served
/// (<https://hl7.org/fhir/R4/http.html#parameters>); a raw `+` in
/// `application/fhir+json` decodes to a space. A repeated parameter's last
/// value counts.
fn pretty_requested(query: &str) -> Result<bool, OperationOutcomeError> {
    let (mut format, mut pretty) = (None, None);
    for (name, value) in url::form_urlencoded::parse(query.as_bytes()) {
        match name.as_ref() {
            "_format" => format = Some(value),
            "_pretty" => pretty = Some(value),
            _ => {}
        }
    }
    if let Some(format) = format.as_deref()
        && !matches!(
            format,
            "json" | "application/json" | "application/fhir+json" | "application/fhir json"
        )
    {
        return Err(OperationOutcomeError::error(
            IssueType::not_supported(),
            format!("_format '{format}' is not supported; only JSON is served."),
        ));
    }
    Ok(pretty.as_deref() == Some("true"))
}

async fn pretty_print(response: Response) -> Result<Response, OperationOutcomeError> {
    let (mut parts, body) = response.into_parts();
    let bytes = to_bytes(body, usize::MAX)
        .await
        .map_err(|e| OperationOutcomeError::error(IssueType::exception(), e.to_string()))?;
    // Not JSON (or unreadable): leave the body as it was.
    let bytes = serde_json::from_slice::<serde_json::Value>(&bytes)
        .and_then(|value| serde_json::to_vec_pretty(&value))
        .map_or(bytes, Bytes::from);
    parts.headers.remove(axum::http::header::CONTENT_LENGTH);
    Ok(Response::from_parts(parts, Body::from(bytes)))
}

async fn fhir_handler<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    method: Method,
    Extension(user): Extension<Arc<User>>,
    uri: Uri,
    OriginalUri(original_uri): OriginalUri,
    Path(path): Path<ProjectPath>,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, OperationOutcomeError> {
    // Nested under the FHIR root, so the request URI is what follows it: the
    // FHIR location, optionally preceded by the version.
    let (requested_version, fhir_location) = split_fhir_version(uri.path());
    let fhir_version = request_fhir_version(&user, &path, requested_version)?;

    let ctx = Arc::new(
        ServerCTX::new(
            path.tenant,
            path.project,
            fhir_version,
            user,
            state.fhir_client.clone(),
            state.rate_limit.clone(),
        )
        .with_tracing_id(Some(format!("rest-{}", generate_id(Some(8)))))
        .with_prefer_async(jobs::prefers_async(&headers)),
    );

    let query = original_uri.query().unwrap_or_default();
    let http_req = HTTPRequest::new(
        method,
        fhir_location.to_string(),
        HTTPBody::Bytes(body),
        query,
    );
    let fhir_request =
        http_request_to_fhir_request(SupportedFHIRVersions::R4, http_req, &ctx).await?;

    // Searches only: `_format` is also an operation parameter.
    let pretty = matches!(fhir_request, FHIRRequest::Search(_)) && pretty_requested(query)?;

    let response = state.fhir_client.request(ctx, fhir_request).await?;

    let response = response.into_response();
    if pretty {
        pretty_print(response).await
    } else {
        Ok(response)
    }
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

    // Everything under the FHIR root is a FHIR location. The version is a
    // property of the project, so it is optional in the URL: `…/fhir/Patient/1`
    // and `…/fhir/r4/Patient/1` are the same request, and the handler takes the
    // version off the front of the location. Keeping that out of the route
    // table means the two cannot go out of step.
    //
    // Two routes only because a wildcard does not match an empty segment, so it
    // cannot match the FHIR root on its own.
    let fhir_router = Router::new()
        .route("/", any(fhir_handler))
        .route("/{*fhir_location}", any(fhir_handler));

    let protected_resources_router = Router::new()
        .nest("/fhir", fhir_router)
        .route("/mcp", post(mcp::route::mcp_handler))
        // The status URL of an asynchronous job: polled, or deleted to cancel.
        .route(
            "/jobs/{id}",
            get(jobs::status_handler).delete(jobs::cancel_handler),
        )
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

    // Per spec must be at root of the fhir server which is why
    // /fhir/.well-known/smart-configuration is used as the route. Because this is
    // publically available it is not under protected_resources_router and does not
    // require authentication.
    let mut smart_configuration_router = Router::new().route(
        "/fhir/.well-known/smart-configuration",
        get(auth_n::oidc::routes::discovery::smart_configuration),
    );

    for version in SupportedFHIRVersions::ALL {
        smart_configuration_router = smart_configuration_router.route(
            &format!("/fhir/{version}/.well-known/smart-configuration"),
            get(auth_n::oidc::routes::discovery::smart_configuration),
        );
    }

    let smart_configuration_router = smart_configuration_router.route_layer(
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
        project_router = project_router.route("/fhir/metadata", get(public_metadata_handler));

        for version in SupportedFHIRVersions::ALL {
            project_router = project_router.route(
                &format!("/fhir/{version}/metadata"),
                get(public_metadata_handler),
            );
        }
    }

    let tenant_router = Router::new()
        .route("/branding/logo", get(auth_n::tenant::routes::logo))
        .nest("/auth", auth_n::tenant::routes::create_router())
        // OpenID Connect Discovery 1.0 §4: the document lives at `[iss]/.well-known/openid-configuration`.
        .route(
            "/{project}/.well-known/openid-configuration",
            get(auth_n::oidc::routes::discovery::openid_configuration),
        )
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
                // Every request gets an id (a client's own is kept) that the
                // error log and the response carry, so a support report can
                // be matched to the log line.
                .layer(SetRequestIdLayer::new(REQUEST_ID_HEADER, MakeRequestUuid))
                .layer(PropagateRequestIdLayer::new(REQUEST_ID_HEADER))
                .layer(CatchPanicLayer::new())
                .layer(ip_source.into_extension())
                .layer(NewSentryLayer::<Request<Body>>::new_from_top())
                .layer(TraceLayer::new_for_http())
                // 4mb by default.
                .layer(DefaultBodyLimit::max(config.max_request_body_size))
                // HTML pages carry the session's CSRF token next to text the
                // caller chose (the query string echoed into form actions,
                // the entered email). Compressing the two together lets the
                // token be recovered byte by byte from response sizes
                // (BREACH), so pages are sent uncompressed; JSON and FHIR
                // responses, which is where size matters, still are.
                .layer(CompressionLayer::new().compress_when(
                    DefaultPredicate::new().and(NotForContentType::new("text/html")),
                ))
                .layer(security_headers(config.signup.turnstile.is_some()))
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
                        .allow_headers(AllowHeaders::mirror_request())
                        // Asynchronous from browser need these headers.
                        .expose_headers([
                            header::CONTENT_LOCATION,
                            header::ETAG,
                            header::RETRY_AFTER,
                            HeaderName::from_static("x-progress"),
                        ])
                        // Set the max age for CORS preflight requests to 2 hours.
                        .max_age(std::time::Duration::from_secs(7200)),
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[test]
    fn format_and_pretty_parameters() {
        for format in [
            "json",
            "application/json",
            "application/fhir%2Bjson",
            "application/fhir+json",
        ] {
            assert!(!pretty_requested(&format!("_format={format}")).unwrap());
        }
        assert!(pretty_requested("name=x&_pretty=true").unwrap());
        assert!(!pretty_requested("_pretty=false").unwrap());
        assert!(!pretty_requested("_pretty=true&_pretty=false").unwrap());
        assert!(pretty_requested("_format=xml").is_err());
    }

    #[test]
    fn version_is_split_off_when_the_url_carries_one() {
        assert_eq!(
            split_fhir_version("/r4/Patient/123"),
            (Some(SupportedFHIRVersions::R4), "Patient/123")
        );
        assert_eq!(
            split_fhir_version("/r4"),
            (Some(SupportedFHIRVersions::R4), "")
        );
    }

    #[test]
    fn a_location_is_not_mistaken_for_a_version() {
        assert_eq!(split_fhir_version("/Patient/123"), (None, "Patient/123"));
        assert_eq!(split_fhir_version("/"), (None, ""));
        // An unsupported version reads as a resource type, and fails as one.
        assert_eq!(split_fhir_version("/r4b/Patient"), (None, "r4b/Patient"));
    }

    /// Mirrors the routes `server()` builds, to check which handler each URL
    /// shape reaches. The FHIR routes answer with and without the version, and
    /// the public routes under the same prefix have to keep matching ahead of
    /// the FHIR wildcard.
    fn router() -> Router {
        // Echoes what `fhir_handler` derives, so the tests check the location
        // it would actually work on and not just which route matched.
        async fn fhir(uri: Uri) -> String {
            let (version, location) = split_fhir_version(uri.path());
            format!(
                "fhir version={} location={location}",
                version.map_or("-".to_string(), |version| version.to_string())
            )
        }

        let fhir_router = Router::new()
            .route("/", any(fhir))
            .route("/{*fhir_location}", any(fhir));

        // Nested and merged exactly as `server()` does it, because how much of
        // the path is left for the handler to read depends on that nesting.
        let protected_resources_router = Router::new().nest("/fhir", fhir_router);

        let mut project_router = Router::new()
            .merge(protected_resources_router)
            .route(
                "/fhir/.well-known/smart-configuration",
                get(|| async { "smart" }),
            )
            .route("/fhir/metadata", get(|| async { "metadata" }));

        for version in SupportedFHIRVersions::ALL {
            project_router = project_router
                .route(
                    &format!("/fhir/{version}/.well-known/smart-configuration"),
                    get(|| async { "smart" }),
                )
                .route(
                    &format!("/fhir/{version}/metadata"),
                    get(|| async { "metadata" }),
                );
        }

        let tenant_router = Router::new().nest("/{project}/api/v1", project_router);

        Router::new().nest("/w/{tenant}", tenant_router)
    }

    async fn route_to(uri: &str) -> String {
        let response = router()
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK, "{uri}");

        String::from_utf8(
            to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap()
    }

    /// Both URL shapes reach the handler and resolve to the same FHIR
    /// location, which also pins that nesting strips the prefix before the
    /// handler reads the URI.
    #[tokio::test]
    async fn both_url_shapes_address_the_same_location() {
        let base = "/w/acme/default/api/v1/fhir";

        for (uri, expected) in [
            (base.to_string(), "fhir version=- location="),
            (format!("{base}/Patient"), "fhir version=- location=Patient"),
            (
                format!("{base}/Patient/123/_history/1"),
                "fhir version=- location=Patient/123/_history/1",
            ),
            (format!("{base}/r4"), "fhir version=r4 location="),
            (
                format!("{base}/r4/Patient"),
                "fhir version=r4 location=Patient",
            ),
            (
                format!("{base}/r4/Patient/123/_history/1"),
                "fhir version=r4 location=Patient/123/_history/1",
            ),
        ] {
            assert_eq!(route_to(&uri).await, expected, "{uri}");
        }
    }

    #[tokio::test]
    async fn public_routes_keep_matching_ahead_of_the_fhir_routes() {
        let base = "/w/acme/default/api/v1/fhir";

        for (uri, expected) in [
            (format!("{base}/metadata"), "metadata"),
            (format!("{base}/r4/metadata"), "metadata"),
            (format!("{base}/.well-known/smart-configuration"), "smart"),
            (
                format!("{base}/r4/.well-known/smart-configuration"),
                "smart",
            ),
        ] {
            assert_eq!(route_to(&uri).await, expected, "{uri}");
        }
    }
}
