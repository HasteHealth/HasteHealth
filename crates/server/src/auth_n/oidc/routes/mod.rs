use std::sync::Arc;

use axum::{Router, extract::Json};
use axum_extra::routing::{
    RouterExt, // for `Router::typed_*`
    TypedPath,
};
use oxidized_fhir_repository::TenantId;
use serde::{Deserialize, Serialize};

// A type safe route with `/users/{id}` as its associated path.
#[derive(TypedPath, Deserialize)]
#[typed_path("/{tenant}/token/{id}")]
pub struct TokenPostRoute {
    tenant: TenantId,
    id: String,
}

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

async fn well_known(_: WellKnown) -> Result<Json<OIDCResponse>, String> {
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

async fn token_post(TokenPostRoute { tenant, id }: TokenPostRoute) -> String {
    id
}

pub fn create_router<T: Send + Sync + 'static>() -> Router<Arc<T>> {
    Router::new()
        .typed_get(token_get)
        .typed_post(token_post)
        .typed_get(well_known)
}
