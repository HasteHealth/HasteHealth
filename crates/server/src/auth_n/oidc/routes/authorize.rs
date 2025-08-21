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
use axum::{
    Extension,
    extract::State,
    http::{Uri, uri::Scheme},
    response::Redirect,
};
use axum_extra::routing::TypedPath;
use oxidized_fhir_operation_error::{OperationOutcomeCodes, OperationOutcomeError};
use oxidized_fhir_search::SearchEngine;
use oxidized_repository::{
    Repository,
    admin::ProjectAuthAdmin,
    types::authorization_code::{
        AuthorizationCodeKind, CreateAuthorizationCode, PKCECodeChallengeMethod,
    },
};
use std::{sync::Arc, time::Duration};
use tower_sessions::Session;

#[derive(TypedPath)]
#[typed_path("/authorize")]
pub struct Authorize;

pub async fn authorize<Repo: Repository + Send + Sync, Search: SearchEngine + Send + Sync>(
    _: Authorize,
    tenant: TenantProject,
    State(app_state): State<Arc<AppState<Repo, Search>>>,
    OIDCClientApplication(client_app): OIDCClientApplication,
    Extension(oidc_params): Extension<OIDCParameters>,
    current_session: Session,
) -> Result<Redirect, OperationOutcomeError> {
    let user = session::user::get_user(current_session).await?.unwrap();
    let state = oidc_params.parameters.get("state").ok_or_else(|| {
        OperationOutcomeError::error(
            OperationOutcomeCodes::Invalid,
            "state parameter is required.".to_string(),
        )
    })?;

    let redirect_uri = oidc_params.parameters.get("redirect_uri").ok_or_else(|| {
        OperationOutcomeError::error(
            OperationOutcomeCodes::Invalid,
            "redirect_uri parameter is required.".to_string(),
        )
    })?;

    let Some(code_challenge) = oidc_params.parameters.get("code_challenge") else {
        return Err(OperationOutcomeError::error(
            OperationOutcomeCodes::Invalid,
            "code_challenge parameter is required.".to_string(),
        ));
    };

    let Some(code_challenge_method) = oidc_params
        .parameters
        .get("code_challenge_method")
        .and_then(|code_challenge_method| {
            PKCECodeChallengeMethod::try_from(code_challenge_method.as_str()).ok()
        })
    else {
        return Err(OperationOutcomeError::error(
            OperationOutcomeCodes::Invalid,
            "code_challenge_method must be a valid PKCE code challenge method.".to_string(),
        ));
    };

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
        &app_state.repo,
        &tenant.tenant,
        &tenant.project,
        CreateAuthorizationCode {
            expires_in: Duration::from_secs(60 * 5),
            kind: AuthorizationCodeKind::OAuth2CodeGrant,
            user_id: user.id,
            client_id: Some(client_id.to_string()),
            pkce_code_challenge: Some(code_challenge.to_string()),
            pkce_code_challenge_method: Some(code_challenge_method),
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
        .scheme(uri.scheme().cloned().unwrap_or(Scheme::HTTPS))
        .authority(uri.authority().unwrap().clone())
        .path_and_query(
            uri.path().to_string() + "?code=" + &authorzation_code.code + "&state=" + state,
        )
        .build()
        .unwrap();

    Ok(Redirect::to(&redirection.to_string()))
}
