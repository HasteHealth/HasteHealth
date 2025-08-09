use std::sync::Arc;

use axum::{
    Router,
    body::Body,
    extract::{Json, Request, State},
    response::Response,
};
use axum_extra::routing::{
    RouterExt, // for `Router::typed_*`
    TypedPath,
};
use oxidized_fhir_client::FHIRClient;
use oxidized_fhir_model::r4::types::ResourceType;
use oxidized_fhir_repository::{Author, FHIRRepository, ProjectId, TenantId};
use oxidized_fhir_search::SearchEngine;
use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;

use crate::{AppState, server_client::ServerCTX};

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

async fn middleware_test<
    Repo: FHIRRepository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
>(
    State(state): State<Arc<AppState<Repo, Search>>>,
    req: Request<Body>,
    next: axum::middleware::Next,
) -> Response<Body> {
    let ctx = ServerCTX {
        tenant: TenantId::new("tenant".to_string()),
        project: ProjectId::new("project".to_string()),
        fhir_version: oxidized_fhir_repository::SupportedFHIRVersions::R4,
        author: Author {
            id: "anonymous".to_string(),
            kind: "Membership".to_string(),
        },
    };

    let res = state
        .fhir_client
        .search_type(
            ctx,
            ResourceType::new("ClientApplication".to_string()).unwrap(),
            vec![],
        )
        .await;
    println!("{:?}", res);

    let response = next.run(req).await;

    // do something with `response`...

    response
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
        .typed_get(well_known)
        .layer(
            ServiceBuilder::new().layer(axum::middleware::from_fn_with_state(
                state.clone(),
                middleware_test,
            )),
        )
}
