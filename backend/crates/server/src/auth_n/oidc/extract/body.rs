use axum::{
    body::to_bytes,
    extract::{FromRequest, Request},
    http::{HeaderMap, header},
};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use serde::de::DeserializeOwned;

use crate::{
    auth_n::oidc::{
        error::{OIDCError, OIDCErrorCode},
        schemas,
    },
    extract::basic_credentials::{BasicCredentials, BasicCredentialsHeader},
};

/// Why an OAuth request body could not be read.
enum BodyError {
    Unreadable,
    Invalid(String),
}

/// Reads `req`'s body as `T`. Supports 'application/json',
/// 'application/fhir+json', and 'application/x-www-form-urlencoded' content
/// types.
async fn parse_body<T: DeserializeOwned>(req: Request) -> Result<(HeaderMap, T), BodyError> {
    let (parts, body) = req.into_parts();
    let content_type = parts
        .headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let bytes = to_bytes(body, 1_000_000)
        .await
        .map_err(|_e| BodyError::Unreadable)?;

    let body = match content_type {
        "application/json" | "application/fhir+json" => serde_json::from_slice::<T>(&bytes)
            .map_err(|e| {
                tracing::error!("JSON parse error: {:?}", e);
                BodyError::Invalid(e.to_string())
            })?,
        "application/x-www-form-urlencoded" => serde_html_form::from_bytes::<T>(&bytes)
            .map_err(|e| BodyError::Invalid(e.to_string()))?,
        _ => {
            return Err(BodyError::Invalid(
                "Invalid content type, expected 'application/json' or 'application/fhir+json'"
                    .to_string(),
            ));
        }
    };

    Ok((parts.headers, body))
}

/// The client credentials sent in a Basic Authorization header
/// (`client_secret_basic`, RFC 6749 §2.3.1), if any.
fn basic_credentials(headers: &HeaderMap) -> Option<BasicCredentials> {
    let header = headers.get(header::AUTHORIZATION)?.to_str().ok()?;

    match BasicCredentialsHeader::from_header(header) {
        Ok(BasicCredentialsHeader(credentials)) => credentials,
        Err(_) => None,
    }
}

/// Extracts and parses the token request body.
#[derive(Debug, Clone)]
pub struct OAuthTokenBody(pub schemas::token_body::OAuth2TokenBody);

impl<S> FromRequest<S> for OAuthTokenBody
where
    S: Send + Sync,
{
    type Rejection = OperationOutcomeError;

    async fn from_request(req: Request, _s: &S) -> Result<Self, Self::Rejection> {
        let (headers, mut token_body) = parse_body::<schemas::token_body::OAuth2TokenBody>(req)
            .await
            .map_err(|e| match e {
                BodyError::Unreadable => OperationOutcomeError::fatal(
                    IssueType::exception(),
                    "Failed to extract request body".to_string(),
                ),
                BodyError::Invalid(message) => {
                    OperationOutcomeError::fatal(IssueType::invalid(), message)
                }
            })?;

        // In event the client credentials were sent in Basic Auth header we insert them here.
        if let Some(BasicCredentials(client_id, client_secret)) = basic_credentials(&headers) {
            token_body.client_id = Some(client_id);
            token_body.client_secret = Some(client_secret);
        }

        Ok(OAuthTokenBody(token_body))
    }
}

/// Extracts and parses a token introspection request body (RFC 7662 §2.1).
/// Client credentials in a Basic Authorization header take the place of
/// any in the body.
#[derive(Debug, Clone)]
pub struct OAuthIntrospectionBody(pub schemas::token_introspection::OAuth2TokenIntrospectionBody);

impl<S> FromRequest<S> for OAuthIntrospectionBody
where
    S: Send + Sync,
{
    type Rejection = OIDCError;

    async fn from_request(req: Request, _s: &S) -> Result<Self, Self::Rejection> {
        let (headers, mut body) =
            parse_body::<schemas::token_introspection::OAuth2TokenIntrospectionBody>(req)
                .await
                .map_err(|e| match e {
                    BodyError::Unreadable => OIDCError::new(
                        OIDCErrorCode::InvalidRequest,
                        Some("Failed to extract request body".to_string()),
                        None,
                    ),
                    BodyError::Invalid(message) => {
                        OIDCError::new(OIDCErrorCode::InvalidRequest, Some(message), None)
                    }
                })?;

        if let Some(BasicCredentials(client_id, client_secret)) = basic_credentials(&headers) {
            body.client_id = Some(client_id);
            body.client_secret = Some(client_secret);
        }

        Ok(OAuthIntrospectionBody(body))
    }
}
