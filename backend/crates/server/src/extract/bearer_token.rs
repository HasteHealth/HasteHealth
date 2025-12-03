use axum::{extract::FromRequestParts, http::request::Parts};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AuthBearer(pub Option<String>);

impl AuthBearer {
    fn from_header(contents: &str) -> Self {
        Self(Some(contents.to_string()))
    }
}

impl<B> FromRequestParts<B> for AuthBearer
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
            Some((name, contents)) if name == "Bearer" => Ok(Self::from_header(contents)),
            // Found empty bearer; sometimes request libraries format them as this
            _ if authorization == "Bearer" => Ok(Self::from_header("")),
            // Found nothing
            _ => Err(OperationOutcomeError::error(
                IssueType::Invalid(None),
                "Invalid Authorization Header".to_string(),
            )),
        }
    }
}
