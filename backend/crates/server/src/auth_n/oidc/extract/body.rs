use axum::{
    body::to_bytes,
    extract::{FromRequest, Request},
};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;

use crate::{auth_n::oidc::schemas, extract::basic_credentials::BasicCredentialsHeader};

/// Extracts and parses the request body into the specified type T.
/// Supports 'application/json', 'application/fhir+json', and 'application/x-www-form-urlencoded' content types.
#[derive(Debug, Clone)]
pub struct OAuthTokenBody(pub schemas::token_body::OAuth2TokenBody);

impl<S> FromRequest<S> for OAuthTokenBody
where
    S: Send + Sync,
{
    type Rejection = OperationOutcomeError;

    async fn from_request(req: Request, _s: &S) -> Result<Self, Self::Rejection> {
        let (parts, body) = req.into_parts();
        let content_type = parts
            .headers
            .get(axum::http::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let bytes = to_bytes(body, 1000000).await.map_err(|_e| {
            OperationOutcomeError::fatal(
                IssueType::EXCEPTION,
                "Failed to extract request body".to_string(),
            )
        })?;

        let mut token_body = match content_type {
            "application/json" | "application/fhir+json" => {
                let body = serde_json::from_slice::<schemas::token_body::OAuth2TokenBody>(&bytes)
                    .map_err(|e| {
                    tracing::error!("JSON parse error: {:?}", e);
                    OperationOutcomeError::fatal(IssueType::INVALID, e.to_string())
                })?;

                body
            }
            "application/x-www-form-urlencoded" => {
                let body =
                    serde_html_form::from_bytes::<schemas::token_body::OAuth2TokenBody>(&bytes)
                        .map_err(|e| {
                            OperationOutcomeError::fatal(IssueType::INVALID, e.to_string())
                        })?;

                body
            }
            _ => {
                return Err(OperationOutcomeError::fatal(
                    haste_fhir_model::r4::generated::terminology::IssueType::INVALID,
                    "Invalid content type, expected 'application/json' or 'application/fhir+json'"
                        .to_string(),
                ));
            }
        };

        // In event the client credentials were sent in Basic Auth header we insert them here.
        if let Some(basic_header) = parts.headers.get(axum::http::header::AUTHORIZATION)
            && let Some(basic_header) = basic_header.to_str().ok()
            && let Ok(BasicCredentialsHeader(Some(basic_credentials))) =
                BasicCredentialsHeader::from_header(basic_header)
        {
            token_body.client_id = Some(basic_credentials.0);
            token_body.client_secret = Some(basic_credentials.1);
        }

        Ok(OAuthTokenBody(token_body))
    }
}
