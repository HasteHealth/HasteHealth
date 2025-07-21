use crate::request::FHIRRequest;
use axum::{
    body::Bytes,
    extract::{FromRequest, Request},
    response::IntoResponse,
};
use http::Method;
use thiserror::Error;

pub struct FHIRRequestExtractor(pub FHIRRequest);

pub struct OperationOutcomeError {
    pub status: u16,
    pub issue: Vec<String>,
}

#[derive(Error, Debug)]
pub enum FHIRRequestError {
    #[error("Failed to extract FHIR request: '{0}'")]
    FailureToExtract(String),
}

impl IntoResponse for FHIRRequestError {
    fn into_response(self) -> http::Response<axum::body::Body> {
        let status = match self {
            FHIRRequestError::FailureToExtract(_) => http::StatusCode::BAD_REQUEST,
        };

        let body = match self {
            FHIRRequestError::FailureToExtract(_msg) => "Invalid request",
        };

        (status, body).into_response()
    }
}

fn extract_resource_body(bytes: Bytes) -> Result<String, FHIRRequestError> {
    String::from_utf8(bytes.to_vec()).map_err(|_| {
        FHIRRequestError::FailureToExtract("Failed to convert bytes to string".to_string())
    })
}

impl<S> FromRequest<S> for FHIRRequestExtractor
where
    S: Send + Sync,
{
    #[doc = " If the extractor fails it\'ll use this \"rejection\" type. A rejection is"]
    #[doc = " a kind of error that can be converted into a response."]
    type Rejection = FHIRRequestError;

    #[doc = " Perform the extraction."]
    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let method = req.method().clone();
        let path = req.uri().path_and_query().unwrap();

        let bytes = Bytes::from_request(req, state)
            .await
            .map_err(|e| FHIRRequestError::FailureToExtract("Method".to_string()))?;

        match method {
            Method::POST => {
                todo!()
            }
            Method::PUT => {
                // let body = extract_resource_body(bytes)?;
                todo!()
                // Ok(FHIRRequestExtractor(FHIRRequest::UpdateInstance(
                //     resource.to_string(),
                // )))
            }
            Method::GET => {
                todo!()
            }
            Method::DELETE => {
                todo!()
            }
            _ => {
                todo!()
            }
        }

        // let body: &str = bytes.into();

        // let resource = Resource::from_str(body);
    }
}
