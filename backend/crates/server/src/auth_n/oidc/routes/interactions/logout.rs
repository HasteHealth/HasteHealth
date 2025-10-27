use axum::{Extension, response::Redirect};
use axum_extra::{extract::Cached, routing::TypedPath};
use oxidized_fhir_model::r4::generated::terminology::IssueType;
use oxidized_fhir_operation_error::OperationOutcomeError;
use tower_sessions::Session;

use crate::{
    auth_n::{
        oidc::{
            extract::client_app::OIDCClientApplication, middleware::OIDCParameters,
            utilities::is_valid_redirect_url,
        },
        session,
    },
    extract::path_tenant::TenantIdentifier,
};

#[derive(TypedPath)]
#[typed_path("/logout")]
pub struct Logout;

pub async fn logout(
    _: Logout,
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    OIDCClientApplication(_client_app): OIDCClientApplication,
    Cached(current_session): Cached<Session>,
    Extension(oidc_params): Extension<OIDCParameters>,
) -> Result<Redirect, OperationOutcomeError> {
    session::user::clear_user(&current_session, &tenant).await?;

    let redirect_uri = oidc_params.parameters.get("redirect_uri").ok_or_else(|| {
        OperationOutcomeError::error(
            IssueType::Invalid(None),
            "redirect_uri parameter is required.".to_string(),
        )
    })?;

    if is_valid_redirect_url(&redirect_uri, &_client_app) {
        Ok(Redirect::to(&redirect_uri.replace("/logout", "/login")))
    } else {
        Err(OperationOutcomeError::error(
            IssueType::Invalid(None),
            "Invalid redirect_uri parameter.".to_string(),
        ))
    }
}
