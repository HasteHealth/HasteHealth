use std::sync::Arc;

use axum::{Extension, Router, extract::Json};
use axum_extra::routing::{
    RouterExt, // for `Router::typed_*`
    TypedPath,
};

use oxidized_fhir_model::r4::types::ClientApplication;
use oxidized_fhir_repository::{FHIRRepository, TenantId};
use oxidized_fhir_search::SearchEngine;
use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;

use crate::{
    AppState,
    auth_n::oidc::{self, middleware::OIDCParameters},
};

// A type safe route with `/users/{id}` as its associated path.
#[derive(TypedPath, Deserialize)]
#[typed_path("/{tenant}/token/{id}")]
pub struct TokenGetRoute {
    tenant: TenantId,
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
    Extension(client_apps): Extension<Vec<ClientApplication>>,
) -> Result<Json<OIDCResponse>, String> {
    let oidc_response = serde_json::from_value::<OIDCResponse>(serde_json::json!({
        "issuer": "https://example.com",
        "authorization_endpoint": "https://example.com/authorize"
    }))
    .map_err(|_| "Failed to create OIDC response".to_string())?;

    Ok(Json(oidc_response))
}

async fn token_get(TokenGetRoute { tenant, id }: TokenGetRoute) -> String {
    id
}

// A type safe route with `/users/{id}` as its associated path.
#[derive(TypedPath, Deserialize)]
#[typed_path("/{tenant}/token/{id}")]
pub struct TokenPostRoute {
    tenant: TenantId,
    id: String,
}
async fn token_post(
    TokenPostRoute { tenant, id }: TokenPostRoute,
    Extension(client_apps): Extension<Vec<ClientApplication>>,
    Extension(oidc_params): Extension<OIDCParameters>,
) -> String {
    println!(
        "Token Post for tenant: {}, id: {}, params: {:?}",
        tenant, id, oidc_params
    );
    id
}

pub fn create_router<
    T: Send + Sync + 'static,
    Repo: FHIRRepository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
>(
    state: Arc<AppState<Repo, Search>>,
) -> Router<Arc<T>> {
    Router::new()
        .typed_get(token_get)
        .typed_post(token_post)
        .route_layer(
            ServiceBuilder::new()
                .layer(axum::middleware::from_fn_with_state(
                    state.clone(),
                    oidc::middleware::parameter_inject_middleware,
                ))
                .layer(axum::middleware::from_fn_with_state(
                    state.clone(),
                    oidc::middleware::client_inject_middleware,
                )),
        )
        .typed_get(well_known)
        .layer(
            ServiceBuilder::new().layer(axum::middleware::from_fn_with_state(
                state.clone(),
                oidc::middleware::client_inject_middleware,
            )),
        )
}
