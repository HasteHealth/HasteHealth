use crate::OperationOutcomeError;
use axum::response::IntoResponse;
use haste_fhir_model::r4::generated::terminology::IssueType;
use std::sync::Arc;

impl OperationOutcomeError {
    pub fn status(&self) -> axum::http::StatusCode {
        match self.outcome.issue.first() {
            Some(issue) => match issue.code.as_ref() {
                IssueType::Invalid(_) => axum::http::StatusCode::BAD_REQUEST,
                IssueType::NotFound(_) => axum::http::StatusCode::NOT_FOUND,
                IssueType::Forbidden(_) => axum::http::StatusCode::FORBIDDEN,
                IssueType::Conflict(_) => axum::http::StatusCode::CONFLICT,
                _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            },
            None => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for OperationOutcomeError {
    fn into_response(self) -> axum::response::Response {
        let status_code = self.status();
        let error = Arc::new(self);
        let outcome = &error.outcome;
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            axum::http::header::CONTENT_TYPE,
            "application/fhir+json".parse().unwrap(),
        );
        let response = haste_fhir_serialization_json::to_string(outcome)
            .expect("Failed to serialize OperationOutcome");

        // Attach the original error to the response extensions for logging middleware to access and content-type handling.
        let mut response = (status_code, headers, response).into_response();
        response.extensions_mut().insert(error);

        response
    }
}
