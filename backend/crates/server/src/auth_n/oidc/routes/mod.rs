use crate::{
    auth_n::oidc::middleware::{
        AuthSessionValidationLayer, OIDCParameterInjectLayer, ParameterConfig, project_exists,
    },
    services::ServerState,
};
use axum::{
    Router,
    http::{
        HeaderValue,
        header::{CACHE_CONTROL, PRAGMA},
    },
    middleware,
};
use axum_extra::routing::RouterExt;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::Repository;
use std::sync::{Arc, LazyLock};
use tower::ServiceBuilder;
use tower_http::set_header::SetResponseHeaderLayer;

mod authorize;
pub mod discovery;
pub mod federated;
pub mod interactions;
mod jwks;
pub mod route_string;
pub mod scope;
pub mod style;
pub mod token;

static AUTHORIZE_PARAMETERS: LazyLock<Arc<ParameterConfig>> = LazyLock::new(|| {
    Arc::new(ParameterConfig {
        required_parameters: vec![
            "client_id".to_string(),
            "response_type".to_string(),
            "state".to_string(),
            "code_challenge".to_string(),
            "code_challenge_method".to_string(),
        ],
        optional_parameters: vec![
            "scope".to_string(),
            "redirect_uri".to_string(),
            // SMART App Launch: the FHIR server the app will call.
            "aud".to_string(),
            // SMART EHR launch: the opaque launch value the app was opened with.
            "launch".to_string(),
        ],
    })
});

static LOGOUT_PARAMETERS: LazyLock<Arc<ParameterConfig>> = LazyLock::new(|| {
    Arc::new(ParameterConfig {
        required_parameters: vec!["client_id".to_string()],
        optional_parameters: vec!["redirect_uri".to_string()],
    })
});

pub fn create_router<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    state: Arc<ServerState<Repo, Search, Terminology>>,
) -> Router<Arc<ServerState<Repo, Search, Terminology>>> {
    Router::new()
        .merge(Router::new().typed_get(jwks::jwks_get))
        .merge(Router::new().typed_get(style::smart_style_get))
        .merge(federated::federated_router())
        .nest(
            "/auth",
            Router::new()
                // Responses that carry tokens must not be cached (RFC 6749
                // section 5.1): Cache-Control for HTTP/1.1 caches and Pragma
                // for HTTP/1.0 ones.
                .merge(
                    Router::new().typed_post(token::token).route_layer(
                        ServiceBuilder::new()
                            .layer(SetResponseHeaderLayer::overriding(
                                CACHE_CONTROL,
                                HeaderValue::from_static("no-store"),
                            ))
                            .layer(SetResponseHeaderLayer::overriding(
                                PRAGMA,
                                HeaderValue::from_static("no-cache"),
                            )),
                    ),
                )
                .merge(
                    Router::new()
                        .merge(
                            Router::new()
                                .typed_post(authorize::authorize)
                                .typed_get(authorize::authorize)
                                .route_layer(ServiceBuilder::new().layer(
                                    OIDCParameterInjectLayer::new((*AUTHORIZE_PARAMETERS).clone()),
                                )),
                        )
                        // The approval form: it posts one `scope` field per
                        // approved scope and reads its own parameters.
                        .merge(Router::new().typed_post(scope::scope_post))
                        .route_layer(
                            ServiceBuilder::new()
                                .layer(AuthSessionValidationLayer::new("interactions/login")),
                        ),
                ),
        )
        .nest("/interactions", interactions::interactions_router())
        .route_layer(
            ServiceBuilder::new().layer(middleware::from_fn_with_state(state, project_exists)),
        )
}
