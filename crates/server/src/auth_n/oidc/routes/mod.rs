use crate::{
    AppState,
    auth_n::oidc::{
        extract::client_app::OIDCClientApplication,
        middleware::{OIDCParameterInjectLayer, OIDCParameters, ParameterConfig},
    },
};
use axum::{Extension, Router, extract::Json};
use axum_extra::routing::{
    RouterExt, // for `Router::typed_*`
    TypedPath,
};
use oxidized_fhir_repository::FHIRRepository;
use oxidized_fhir_search::SearchEngine;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower::ServiceBuilder;

// A type safe route with `/users/{id}` as its associated path.
#[derive(TypedPath, Deserialize)]
#[typed_path("/token/{id}")]
pub struct TokenGetRoute {
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OIDCResponse {
    pub issuer: String,
    pub authorization_endpoint: String,
}

#[derive(TypedPath)]
#[typed_path("/.well-known/openid-configuration")]
pub struct WellKnown;

async fn well_known(
    _: WellKnown,
    Extension(oidc_params): Extension<OIDCParameters>,
) -> Result<Json<OIDCResponse>, String> {
    println!("OIDC Parameters: {:?}", oidc_params.parameters);
    let oidc_response = serde_json::from_value::<OIDCResponse>(serde_json::json!({
        "issuer": "https://example.com",
        "authorization_endpoint": "https://example.com/authorize"
    }))
    .map_err(|_| "Failed to create OIDC response".to_string())?;

    Ok(Json(oidc_response))
}

async fn token_get(route: TokenGetRoute) -> String {
    route.id
}

// A type safe route with `/users/{id}` as its associated path.
#[derive(TypedPath, Deserialize)]
#[typed_path("/token/{id}")]
pub struct TokenPostRoute {
    id: String,
}
async fn token_post(
    TokenPostRoute { id }: TokenPostRoute,
    OIDCClientApplication(client_app): OIDCClientApplication,
    Extension(oidc_params): Extension<OIDCParameters>,
) -> String {
    println!(
        "Token Post for id: {}, params: {:?} client_app: {:?}",
        id, oidc_params.parameters, client_app
    );

    id
}

pub fn create_router<
    Repo: FHIRRepository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
>() -> Router<Arc<AppState<Repo, Search>>> {
    let token_routes = Router::new()
        .typed_get(token_get)
        .typed_post(token_post)
        .route_layer(
            ServiceBuilder::new().layer(OIDCParameterInjectLayer::new(ParameterConfig {
                // Initialize with your desired parameters
                required_parameters: vec!["client_id".to_string()],
                // required_parameters: vec!["param1".to_string(), "param2".to_string()],
                optional_parameters: vec!["optional1".to_string(), "optional2".to_string()],
                allow_launch_parameters: true,
            })),
        );

    let well_known_routes =
        Router::new()
            .typed_get(well_known)
            .route_layer(ServiceBuilder::new().layer(OIDCParameterInjectLayer::new(
                ParameterConfig {
                    // Initialize with your desired parameters
                    required_parameters: vec!["response_type".to_string()],
                    // required_parameters: vec!["param1".to_string(), "param2".to_string()],
                    optional_parameters: vec!["optional1".to_string(), "optional2".to_string()],
                    allow_launch_parameters: true,
                },
            )));

    Router::new().merge(token_routes).merge(well_known_routes)
}
