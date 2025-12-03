use crate::{
    auth_n::oidc::{
        error::{OIDCError, OIDCErrorCode},
        routes::{AUTH_NESTED_PATH, authorize, jwks, token},
    },
    extract::path_tenant::{ProjectIdentifier, TenantIdentifier},
    services::AppState,
};
use axum::extract::{Json, State};
use axum_extra::{extract::Cached, routing::TypedPath};
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, TenantId};
use haste_repository::Repository;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use url::Url;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WellKnownDiscoveryDocument {
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

fn construct_oidc_route(tenant: &TenantId, project: &ProjectId, path: &str) -> String {
    format!(
        "/w/{}/{}/api/v1/oidc{}",
        tenant.as_ref(),
        project.as_ref(),
        path
    )
}

pub async fn openid_configuration<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    Cached(ProjectIdentifier { project }): Cached<ProjectIdentifier>,
    State(state): State<Arc<AppState<Repo, Search, Terminology>>>,
) -> Result<Json<WellKnownDiscoveryDocument>, OIDCError> {
    let api_url_string = state
        .config
        .get(crate::ServerEnvironmentVariables::APIURI)
        .unwrap_or_default();

    if api_url_string.is_empty() {
        return Err(OIDCError::new(
            OIDCErrorCode::ServerError,
            Some("API_URL is not set in the configuration".to_string()),
            None,
        ));
    }

    let Ok(api_url) = Url::parse(&api_url_string) else {
        return Err(OIDCError::new(
            OIDCErrorCode::ServerError,
            Some("Invalid API_URL format".to_string()),
            None,
        ));
    };

    let authorize_path = construct_oidc_route(
        &tenant,
        &project,
        &(AUTH_NESTED_PATH.to_string() + authorize::AuthorizePath.to_string().as_str()),
    );

    let token_path = construct_oidc_route(
        &tenant,
        &project,
        &(AUTH_NESTED_PATH.to_string() + token::TokenPath.to_string().as_str()),
    );

    let jwks_path = construct_oidc_route(
        &tenant,
        &project,
        &(AUTH_NESTED_PATH.to_string() + jwks::JWKSPath.to_string().as_str()),
    );

    let oidc_response = WellKnownDiscoveryDocument {
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
