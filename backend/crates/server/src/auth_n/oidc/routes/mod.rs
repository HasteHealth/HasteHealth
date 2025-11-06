use crate::{
    auth_n::oidc::middleware::{
        AuthSessionValidationLayer, OIDCParameterInjectLayer, ParameterConfig, project_exists,
    },
    services::AppState,
};
use axum::{
    Router,
    extract::{Json, OriginalUri, State},
    middleware,
};
use axum_extra::routing::{RouterExt, TypedPath};
use oxidized_fhir_model::r4::generated::terminology::IssueType;
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_repository::Repository;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, LazyLock};
use tower::ServiceBuilder;
use url::Url;

mod authorize;
mod federated;
mod interactions;
mod jwks;
pub mod route_string;
pub mod scope;
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

static AUTH_NESTED_PATH: &str = "/auth";

async fn openid_configuration<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: WellKnown,
    OriginalUri(uri): OriginalUri,
    State(state): State<Arc<AppState<Repo, Search, Terminology>>>,
) -> Result<Json<OIDCResponse>, OperationOutcomeError> {
    let api_url_string = state
        .config
        .get(crate::ServerEnvironmentVariables::APIURI)
        .unwrap_or_default();

    if api_url_string.is_empty() {
        return Err(OperationOutcomeError::error(
            IssueType::Exception(None),
            "API_URL is not set in the configuration".to_string(),
        ));
    }

    let Ok(api_url) = Url::parse(&api_url_string) else {
        return Err(OperationOutcomeError::error(
            IssueType::Exception(None),
            "Invalid API_URL format".to_string(),
        ));
    };

    let path = uri.path();
    let well_known_path = WellKnown.to_string();

    let authorize_path = path.replace(
        &well_known_path,
        &(AUTH_NESTED_PATH.to_string() + authorize::AuthorizePath.to_string().as_str()),
    );
    let token_path = path.replace(
        &well_known_path,
        &(AUTH_NESTED_PATH.to_string() + token::TokenPath.to_string().as_str()),
    );
    let jwks_path = path.replace(&well_known_path, jwks::JWKSPath.to_string().as_str());

    let oidc_response = OIDCResponse {
        issuer: api_url.to_string(),
        authorization_endpoint: api_url.join(&authorize_path).unwrap().to_string(),
        token_endpoint: api_url.join(&token_path).unwrap().to_string(),
        jwks_uri: api_url.join(&jwks_path).unwrap().to_string(),
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
        .merge(Router::new().typed_get(openid_configuration))
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
