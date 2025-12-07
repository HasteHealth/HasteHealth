use axum::{extract::FromRequestParts, http::request::Parts};
use base64::prelude::*;
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BasicCredentials(pub String, pub String);

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BasicCredentialsHeader(pub Option<BasicCredentials>);

impl BasicCredentialsHeader {
    fn from_header(contents: &str) -> Result<Self, OperationOutcomeError> {
        let decoded_content = PAD.decode(contents);
        let parts = decoded_content.split(":").collect::<Vec<&str>>();

        if parts.len() != 2 {
            return Err(OperationOutcomeError::error(
                IssueType::Invalid(None),
                "Invalid Basic Authorization Header".to_string(),
            ));
        }
        // Should be safe as verified length above.
        Ok(Self(Some(BasicCredentials(
            parts[0].to_string(),
            parts[1].to_string(),
        ))))
    }
}

impl<B> FromRequestParts<B> for BasicCredentialsHeader
where
    B: Send + Sync,
{
    type Rejection = OperationOutcomeError;

    async fn from_request_parts(req: &mut Parts, _: &B) -> Result<Self, OperationOutcomeError> {
        // Get authorization header
        let Some(header) = req.headers.get(axum::http::header::AUTHORIZATION) else {
            return Ok(Self(None));
        };

        let authorization = header.to_str().map_err(|_| {
            OperationOutcomeError::error(
                IssueType::Invalid(None),
                "Invalid Authorization Header".to_string(),
            )
        })?;

        // Check that its a well-formed bearer and return
        let split = authorization.split_once(' ');
        match split {
            // Found proper bearer
            Some((name, contents)) if name == "Basic" => Ok(Self::from_header(contents)?),
            // Found nothing
            _ => Ok(Self(None)),
        }
    }
}
