use axum::{extract::OriginalUri, response::Redirect};
use axum_extra::routing::TypedPath;
use oxidized_fhir_operation_error::OperationOutcomeError;
use tower_sessions::Session;

use crate::auth_n::session;

#[derive(TypedPath)]
#[typed_path("/logout")]
pub struct Logout;

pub async fn logout(
    _: Logout,
    uri: OriginalUri,
    current_session: Session,
) -> Result<Redirect, OperationOutcomeError> {
    session::user::clear_user(current_session).await?;
    let path = uri.path().to_string();
    Ok(Redirect::to(&path.replace("/logout", "/login")))
}
