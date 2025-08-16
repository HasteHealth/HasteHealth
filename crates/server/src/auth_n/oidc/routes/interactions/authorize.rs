use crate::{
    AppState,
    auth_n::{oidc::middleware::OIDCParameters, session},
    extract::path_tenant::TenantProject,
};
use axum::{Extension, extract::State, http::Uri, response::Redirect};
use axum_extra::routing::TypedPath;
use oxidized_fhir_operation_error::OperationOutcomeError;
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

pub async fn authorize<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
>(
    _: Authorize,
    tenant: TenantProject,
    State(state): State<Arc<AppState<Repo, Search>>>,
    Extension(oidc_params): Extension<OIDCParameters>,
    current_session: Session,
) -> Result<Redirect, OperationOutcomeError> {
    let user = session::user::get_user(current_session).await?.unwrap();
    let redirect_uri = oidc_params.parameters.get("redirect_uri").unwrap();
    let client_id = oidc_params.parameters.get("client_id").unwrap();

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
        OperationOutcomeError::error("invalid".to_string(), "Invalid redirect uri".to_string())
    })?;

    let redirection = Uri::builder()
        .scheme("https")
        .authority(uri.authority().unwrap().clone())
        .path_and_query(uri.path().to_string() + "?code=" + &authorzation_code.code)
        .build()
        .unwrap();

    Ok(Redirect::to(&redirection.to_string()))
}
