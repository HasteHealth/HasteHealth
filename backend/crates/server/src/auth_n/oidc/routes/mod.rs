use crate::{
    auth_n::oidc::middleware::{
        AuthSessionValidationLayer, OIDCParameterInjectLayer, ParameterConfig, project_exists,
    },
    services::AppState,
};
use axum::{Router, middleware};
use axum_extra::routing::RouterExt;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::Repository;
use std::sync::{Arc, LazyLock};
use tower::ServiceBuilder;

mod authorize;
pub mod discovery;
pub mod federated;
mod interactions;
mod jwks;
pub mod route_string;
pub mod scope;
mod token;

pub static AUTH_NESTED_PATH: &str = "/auth";

static AUTHORIZE_PARAMETERS: LazyLock<Arc<ParameterConfig>> = LazyLock::new(|| {
    Arc::new(ParameterConfig {
        required_parameters: vec![
            "client_id".to_string(),
            "response_type".to_string(),
            "state".to_string(),
            "code_challenge".to_string(),
            "code_challenge_method".to_string(),
        ],
        optional_parameters: vec!["scope".to_string(), "redirect_uri".to_string()],
        allow_launch_parameters: true,
    })
});

static LOGOUT_PARAMETERS: LazyLock<Arc<ParameterConfig>> = LazyLock::new(|| {
    Arc::new(ParameterConfig {
        required_parameters: vec!["client_id".to_string()],
        optional_parameters: vec!["redirect_uri".to_string()],
        allow_launch_parameters: true,
    })
});

pub fn create_router<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    state: Arc<AppState<Repo, Search, Terminology>>,
) -> Router<Arc<AppState<Repo, Search, Terminology>>> {
    Router::new()
        .merge(Router::new().typed_get(jwks::jwks_get))
        .merge(federated::federated_router())
        .nest(
            AUTH_NESTED_PATH,
            Router::new()
                .merge(Router::new().typed_post(token::token))
                .merge(
                    Router::new()
                        .merge(
                            Router::new()
                                .typed_post(authorize::authorize)
                                .typed_get(authorize::authorize)
                                .typed_post(scope::scope_post)
                                .route_layer(ServiceBuilder::new().layer(
                                    OIDCParameterInjectLayer::new((*AUTHORIZE_PARAMETERS).clone()),
                                )),
                        )
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
