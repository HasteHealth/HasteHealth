use std::sync::Arc;

use axum::{
    extract::{OriginalUri, Query, State},
    response::Redirect,
};
use axum_extra::{extract::Cached, routing::TypedPath};
use oxidized_fhir_model::r4::generated::terminology::IssueType;
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_repository::Repository;
use serde::{Deserialize, Serialize};
use tower_sessions::Session;
use url::Url;

use crate::{
    auth_n::oidc::routes::federated::initiate::{get_idp_session_info, validate_and_get_idp},
    extract::path_tenant::{Project, TenantIdentifier},
    services::AppState,
};

#[derive(TypedPath, Deserialize)]
#[typed_path("/federated/{identity_provider_id}/callback")]
pub struct FederatedInitiate {
    pub identity_provider_id: String,
}

#[derive(Serialize)]
enum GrantType {
    #[serde(rename = "authorization_code")]
    AuthorizationCode,
}

#[derive(Serialize)]
struct AuthorizationCodeBody {
    pub grant_type: GrantType,
    pub code: String,
    pub redirect_uri: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub code_verifier: Option<String>,
}

#[derive(Deserialize)]
struct CallbackQueryParams {
    pub code: String,
    pub state: String,
}

pub async fn federated_callback<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    FederatedInitiate {
        identity_provider_id,
    }: FederatedInitiate,
    Query(CallbackQueryParams { code, state }): Query<CallbackQueryParams>,
    State(app_state): State<Arc<AppState<Repo, Search, Terminology>>>,
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    Cached(Project(project_resource)): Cached<Project>,
    Cached(session): Cached<Session>,
) -> Result<Redirect, OperationOutcomeError> {
    let identity_provider = validate_and_get_idp(
        tenant,
        app_state.fhir_client.clone(),
        &project_resource,
        identity_provider_id,
    )
    .await?;

    let client_id = identity_provider
        .oidc
        .as_ref()
        .map(|oidc| oidc.client.clientId.as_ref())
        .and_then(|c| c.value.as_ref())
        .ok_or_else(|| {
            OperationOutcomeError::error(
                IssueType::Invalid(None),
                "Identity Provider is missing client ID".to_string(),
            )
        })?;

    let client_secret = identity_provider
        .oidc
        .as_ref()
        .and_then(|oidc| oidc.client.secret.as_ref())
        .and_then(|secret| secret.value.as_ref());

    let idp_session_info = get_idp_session_info(&session, &identity_provider).await?;

    let code_body = AuthorizationCodeBody {
        grant_type: GrantType::AuthorizationCode,
        code: code,
        redirect_uri: idp_session_info.redirect_uri,
        client_id: client_id.clone(),
        client_secret: client_secret.cloned(),
        code_verifier: idp_session_info.code_verifier,
    };

    Ok(())
}

pub fn create_federated_callback_url(
    api_url_string: &str,
    uri: &OriginalUri,
    idp_id: &str,
    replace_path: &str,
) -> Result<String, OperationOutcomeError> {
    let Ok(api_url) = Url::parse(&api_url_string) else {
        return Err(OperationOutcomeError::error(
            IssueType::Exception(None),
            "Invalid API_URL format".to_string(),
        ));
    };

    let path = uri.path().to_string().replace(
        replace_path,
        &FederatedInitiate {
            identity_provider_id: idp_id.to_string(),
        }
        .to_string(),
    );

    Ok(api_url.join(&path).unwrap().to_string())
}
