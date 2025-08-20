use crate::{
    AppState, auth_n::oidc::middleware::ParameterConfig, extract::path_tenant::TenantProject,
};
use axum::{
    Router,
    extract::{Json, State},
    http::Uri,
};
use axum_extra::routing::{
    RouterExt, // for `Router::typed_*`
    TypedPath,
};
use oxidized_fhir_operation_error::{OperationOutcomeCodes, OperationOutcomeError};
use oxidized_fhir_search::SearchEngine;
use oxidized_repository::Repository;
use serde::{Deserialize, Serialize};
use std::{
    str::FromStr,
    sync::{Arc, LazyLock},
};
use url::Url;

mod interactions;
mod token;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OIDCResponse {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub jwks_uri: String,
}

#[derive(TypedPath)]
#[typed_path("/.well-known/openid-configuration")]
pub struct WellKnown;

async fn openid_configuration<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
>(
    _: WellKnown,
    State(state): State<Arc<AppState<Repo, Search>>>,
    TenantProject { tenant, project }: TenantProject,
) -> Result<Json<OIDCResponse>, OperationOutcomeError> {
    let api_url_string = state.config.get("API_URL").unwrap_or_default();

    if api_url_string.is_empty() {
        return Err(OperationOutcomeError::error(
            OperationOutcomeCodes::Exception,
            "API_URL is not set in the configuration".to_string(),
        ));
    }

    let Ok(api_url) = Url::parse(&api_url_string) else {
        return Err(OperationOutcomeError::error(
            OperationOutcomeCodes::Exception,
            "Invalid API_URL format".to_string(),
        ));
    };

    let oidc_response = OIDCResponse {
        issuer: api_url.to_string(),
        authorization_endpoint: "https://example.com/authorize".to_string(),
        jwks_uri: api_url.join("certs/jwks").unwrap().to_string(),
    };

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

pub fn create_router<Repo: Repository + Send + Sync, Search: SearchEngine + Send + Sync>()
-> Router<Arc<AppState<Repo, Search>>> {
    let well_known_routes = Router::new().typed_get(openid_configuration);

    let token_routes = Router::new().typed_post(token::token);

    Router::new()
        .merge(token_routes)
        .merge(well_known_routes)
        .nest("/interactions", interactions::interactions_router())
}
