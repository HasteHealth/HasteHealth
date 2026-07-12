use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::Cached;

use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_repository::utilities::generate_id;
use tower_sessions::Session;

/// CSRF Token that's stored per user session
/// This token is used to protect against (CSRF) attacks.
pub struct CSRFToken(pub String);

static CSRF_TOKEN_SESSION_KEY: &str = "csrf_token";

impl<B> FromRequestParts<B> for CSRFToken
where
    B: Send + Sync,
{
    type Rejection = OperationOutcomeError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &B,
    ) -> Result<Self, OperationOutcomeError> {
        let Cached(session) = Cached::<Session>::from_request_parts(parts, state)
            .await
            .map_err(|_err| {
                OperationOutcomeError::error(
                    IssueType::Invalid(None),
                    format!("Failed to extract session"),
                )
            })?;

        if let Some(csrf_token) = session
            .get::<String>(CSRF_TOKEN_SESSION_KEY)
            .await
            .map_err(|_e| {
                OperationOutcomeError::error(
                    IssueType::Invalid(None),
                    "Failed to retrieve CSRF Token from session".to_string(),
                )
            })?
        {
            Ok(CSRFToken(csrf_token))
        } else {
            let value = generate_id(Some(32));
            session
                .insert(CSRF_TOKEN_SESSION_KEY, value.clone())
                .await
                .map_err(|_e| {
                    OperationOutcomeError::error(
                        IssueType::Invalid(None),
                        "Failed to insert CSRF Token into session".to_string(),
                    )
                })?;

            Ok(CSRFToken(value))
        }
    }
}
