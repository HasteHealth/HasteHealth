use crate::{
    AppState,
    auth_n::{
        oidc::{
            extract::client_app::OIDCClientApplication, middleware::OIDCParameters,
            utilities::is_valid_redirect_url,
        },
        session,
    },
    extract::path_tenant::TenantProject,
};
use axum::{Extension, extract::State, http::Uri, response::Redirect};
use axum_extra::routing::TypedPath;
use oxidized_fhir_operation_error::{OperationOutcomeCodes, OperationOutcomeError};
use oxidized_fhir_search::SearchEngine;
use oxidized_repository::{
    Repository,
    admin::ProjectAuthAdmin,
    types::authorization_code::{AuthorizationCodeKind, CreateAuthorizationCode},
};
use std::{sync::Arc, time::Duration};
use tower_sessions::Session;

#[derive(TypedPath)]
#[typed_path("/authorize")]
pub struct Authorize;

pub async fn authorize<Repo: Repository + Send + Sync, Search: SearchEngine + Send + Sync>(
    _: Authorize,
    tenant: TenantProject,
    State(state): State<Arc<AppState<Repo, Search>>>,
    OIDCClientApplication(client_app): OIDCClientApplication,
    Extension(oidc_params): Extension<OIDCParameters>,
    current_session: Session,
) -> Result<Redirect, OperationOutcomeError> {
    let user = session::user::get_user(current_session).await?.unwrap();
    let redirect_uri = oidc_params.parameters.get("redirect_uri").ok_or_else(|| {
        OperationOutcomeError::error(
            OperationOutcomeCodes::Invalid,
            "redirect_uri parameter is required.".to_string(),
        )
    })?;

    if !is_valid_redirect_url(&redirect_uri, &client_app) {
        return Err(OperationOutcomeError::error(
            OperationOutcomeCodes::Invalid,
            "Invalid redirect URI.".to_string(),
        ));
    }

    let client_id = client_app.id.ok_or_else(|| {
        OperationOutcomeError::error(
            OperationOutcomeCodes::Invalid,
            "Client ID is required.".to_string(),
        )
    })?;

    let authorzation_code = ProjectAuthAdmin::create(
        &state.repo,
        &tenant.tenant,
        &tenant.project,
        CreateAuthorizationCode {
            expires_in: Duration::from_secs(60 * 5),
            kind: AuthorizationCodeKind::OAuth2CodeGrant,
            user_id: user.id,
            client_id: Some(client_id.to_string()),
            pkce_code_challenge: None,
            pkce_code_challenge_method: None,
            redirect_uri: Some(redirect_uri.to_string()),
            meta: None,
        },
    )
    .await?;

    let uri = Uri::try_from(redirect_uri).map_err(|_| {
        OperationOutcomeError::error(
            OperationOutcomeCodes::Invalid,
            "Invalid redirect uri".to_string(),
        )
    })?;

    let redirection = Uri::builder()
        .scheme("https")
        .authority(uri.authority().unwrap().clone())
        .path_and_query(uri.path().to_string() + "?code=" + &authorzation_code.code)
        .build()
        .unwrap();

    Ok(Redirect::to(&redirection.to_string()))
}
