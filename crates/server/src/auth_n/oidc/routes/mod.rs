use crate::{
    AppState,
    auth_n::oidc::middleware::{OIDCParameterInjectLayer, ParameterConfig},
    extract::path_tenant::TenantProject,
};
use axum::{
    Router,
    extract::{Json, State},
};
use axum_extra::routing::{
    RouterExt, // for `Router::typed_*`
    TypedPath,
};
use oxidized_fhir_operation_error::{OperationOutcomeCodes, OperationOutcomeError};
use oxidized_fhir_search::SearchEngine;
use oxidized_repository::Repository;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, LazyLock};
use tower::ServiceBuilder;
use url::Url;

mod authorize;
mod interactions;
mod token;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OIDCResponse {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub jwks_uri: String,
    pub token_endpoint: String,
    pub scopes_supported: Vec<String>,
    pub response_types_supported: Vec<String>,
    pub token_endpoint_auth_methods_supported: Vec<String>,
    pub id_token_signing_alg_values_supported: Vec<String>,
    pub subject_types_supported: Vec<String>,
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
        authorization_endpoint: api_url.join("auth/authorize").unwrap().to_string(),
        token_endpoint: api_url.join("auth/token").unwrap().to_string(),
        jwks_uri: api_url.join("certs/jwks").unwrap().to_string(),
        scopes_supported: vec![
            "openid".to_string(),
            "profile".to_string(),
            "email".to_string(),
            "offline_access".to_string(),
        ],
        response_types_supported: vec![
            "code".to_string(),
            "id_token".to_string(),
            "id_token token".to_string(),
        ],
        token_endpoint_auth_methods_supported: vec![
            "client_secret_basic".to_string(),
            "client_secret_post".to_string(),
        ],
        id_token_signing_alg_values_supported: vec!["RS256".to_string()],
        subject_types_supported: vec!["public".to_string()],
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

    let authorize_routes = Router::new()
        .typed_post(authorize::authorize)
        .typed_get(authorize::authorize)
        .route_layer(ServiceBuilder::new().layer(OIDCParameterInjectLayer::new(
            (*AUTHORIZE_PARAMETERS).clone(),
        )));

    let auth_router = Router::new().merge(token_routes).merge(authorize_routes);

    Router::new()
        .merge(well_known_routes)
        .nest("/auth", auth_router)
        .nest("/interactions", interactions::interactions_router())
}
