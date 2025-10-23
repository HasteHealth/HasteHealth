use crate::{
    auth_n::oidc::ui::pages,
    extract::path_tenant::{Project, TenantIdentifier},
};
use axum::extract::OriginalUri;
use axum_extra::routing::TypedPath;
use maud::Markup;
use oxidized_fhir_operation_error::OperationOutcomeError;
use serde::Deserialize;

#[derive(TypedPath)]
#[typed_path("/password-reset")]
pub struct Login;

pub async fn password_reset_get(
    _: Login,
    TenantIdentifier { tenant }: TenantIdentifier,
    Project(project): Project,
    _uri: OriginalUri,
) -> Result<Markup, OperationOutcomeError> {
    let response = pages::email_form::email_form_html(
        &tenant,
        &project,
        &pages::email_form::EmailInformation {
            continue_url: "".to_string(),
        },
    );

    Ok(response)
}

#[allow(unused)]
#[derive(Deserialize)]
pub struct PasswordResetFormInitiate {
    pub email: String,
}
