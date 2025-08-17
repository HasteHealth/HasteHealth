use crate::{
    AppState,
    auth_n::oidc::middleware::{OIDCParameterInjectLayer, OIDCParameters, ParameterConfig},
};
use axum::{Extension, Router, extract::Json};
use axum_extra::routing::{
    RouterExt, // for `Router::typed_*`
    TypedPath,
};
use oxidized_fhir_search::SearchEngine;
use oxidized_repository::Repository;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, LazyLock};
use tower::ServiceBuilder;

mod interactions;
mod token;

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

pub fn create_router<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
>() -> Router<Arc<AppState<Repo, Search>>> {
    let well_known_routes =
        Router::new()
            .typed_get(well_known)
            .route_layer(
                ServiceBuilder::new().layer(OIDCParameterInjectLayer::new(Arc::new(
                    ParameterConfig {
                        // Initialize with your desired parameters
                        required_parameters: vec!["response_type".to_string()],
                        // required_parameters: vec!["param1".to_string(), "param2".to_string()],
                        optional_parameters: vec!["optional1".to_string(), "optional2".to_string()],
                        allow_launch_parameters: true,
                    },
                ))),
            );

    let token_routes = Router::new().typed_post(token::token);

    Router::new()
        .merge(token_routes)
        .merge(well_known_routes)
        .nest("/interactions", interactions::interactions_router())
}
