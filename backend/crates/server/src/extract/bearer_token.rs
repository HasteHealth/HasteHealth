use axum::{extract::FromRequestParts, http::request::Parts};
use oxidized_fhir_model::r4::generated::terminology::IssueType;
use oxidized_fhir_operation_error::OperationOutcomeError;

static AUTHORIZATION_HEADER: &str = "Authorization";

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AuthBearer(pub String);

impl AuthBearer {
    fn from_header(contents: &str) -> Self {
        Self(contents.to_string())
    }
}

impl<B> FromRequestParts<B> for AuthBearer
where
    B: Send + Sync,
{
    type Rejection = OperationOutcomeError;

    async fn from_request_parts(req: &mut Parts, _: &B) -> Result<Self, OperationOutcomeError> {
        // Get authorization header
        let authorization = req
            .headers
            .get(AUTHORIZATION_HEADER)
            .ok_or(OperationOutcomeError::error(
                IssueType::Invalid(None),
                "Missing Authorization Header".to_string(),
            ))?
            .to_str()
            .map_err(|_| {
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
