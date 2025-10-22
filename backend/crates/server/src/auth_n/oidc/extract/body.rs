use axum::{
    body::to_bytes,
    extract::{FromRequest, Request},
};
use oxidized_fhir_model::r4::generated::terminology::IssueType;
use oxidized_fhir_operation_error::OperationOutcomeError;
use serde::de::DeserializeOwned;

/// Extracts and parses the request body into the specified type T.
/// Supports 'application/json', 'application/fhir+json', and 'application/x-www-form-urlencoded' content types.
#[derive(Debug, Clone, Copy, Default)]
pub struct ParsedBody<T>(pub T);

impl<T, S> FromRequest<S> for ParsedBody<T>
where
    T: DeserializeOwned,
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
                IssueType::Exception(None),
                "Failed to extract request body".to_string(),
            )
        })?;

        match content_type {
            "application/json" | "application/fhir+json" => {
                let body = serde_json::from_slice::<T>(&bytes).map_err(|e| {
                    println!("JSON parse error: {:?}", e);
                    OperationOutcomeError::fatal(IssueType::Invalid(None), e.to_string())
                })?;

                Ok(ParsedBody(body))
            }
            "application/x-www-form-urlencoded" => {
                let body = serde_html_form::from_bytes::<T>(&bytes).map_err(|e| {
                    OperationOutcomeError::fatal(IssueType::Invalid(None), e.to_string())
                })?;

                Ok(ParsedBody(body))
            }
            _ => {
                return Err(OperationOutcomeError::fatal(
                    oxidized_fhir_model::r4::generated::terminology::IssueType::Invalid(None),
                    "Invalid content type, expected 'application/json' or 'application/fhir+json'"
                        .to_string(),
                ));
            }
        }
    }
}
