use crate::OperationOutcomeError;
use axum::response::IntoResponse;
use haste_fhir_model::r4::generated::terminology::IssueType;
use std::sync::Arc;

impl OperationOutcomeError {
    pub fn status(&self) -> axum::http::StatusCode {
        match self.outcome.issue.first() {
            Some(issue) => {
                if issue.code == IssueType::INVALID {
                    axum::http::StatusCode::BAD_REQUEST
                } else if issue.code == IssueType::NOTFOUND {
                    axum::http::StatusCode::NOT_FOUND
                } else if issue.code == IssueType::FORBIDDEN {
                    axum::http::StatusCode::FORBIDDEN
                } else if issue.code == IssueType::CONFLICT {
                    axum::http::StatusCode::CONFLICT
                } else if issue.code == IssueType::THROTTLED {
                    axum::http::StatusCode::TOO_MANY_REQUESTS
                } else {
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR
                }
            }
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
        let response =
            serde_json::to_string(outcome).expect("Failed to serialize OperationOutcome");

        // Attach the original error to the response extensions for logging middleware to access and content-type handling.
        let mut response = (status_code, headers, response).into_response();
        response.extensions_mut().insert(error);

        response
    }
}
