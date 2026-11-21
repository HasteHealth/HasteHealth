use crate::OperationOutcomeError;
use axum::{
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use std::sync::Arc;

impl OperationOutcomeError {
    /// [`http_status`](Self::http_status) as a `StatusCode`.
    #[must_use]
    pub fn status(&self) -> StatusCode {
        StatusCode::from_u16(self.http_status()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl IntoResponse for OperationOutcomeError {
    fn into_response(self) -> Response {
        let status = self.status();
        let body = serde_json::to_string(self.outcome()).expect("an OperationOutcome serializes");
        let mut response = (
            status,
            [(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/fhir+json"),
            )],
            body,
        )
            .into_response();
        // The logging and content negotiation middleware read the error back from here.
        response.extensions_mut().insert(Arc::new(self));
        response
    }
}
