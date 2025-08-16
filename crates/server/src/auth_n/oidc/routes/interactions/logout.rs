use axum_extra::routing::TypedPath;
use maud::{Markup, html};
use oxidized_fhir_operation_error::OperationOutcomeError;
use tower_sessions::Session;

use crate::auth_n::session;

#[derive(TypedPath)]
#[typed_path("/logout")]
pub struct Logout;

pub async fn logout(_: Logout, current_session: Session) -> Result<Markup, OperationOutcomeError> {
    session::user::clear_user(current_session).await?;
    Ok(html! {
        p { "YOU've logged out."}
    })
}
