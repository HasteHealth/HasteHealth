use std::error::Error;

use axum::response::IntoResponse;
use oxidized_fhir_model::r4::generated::terminology::IssueType;

use crate::OperationOutcomeError;

impl IntoResponse for OperationOutcomeError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("OperationOutcomeError source: {:?}", &self.source());
        let outcome = self.outcome;
        let response = oxidized_fhir_serialization_json::to_string(&outcome)
            .expect("Failed to serialize OperationOutcome");

        let status_code = match outcome.issue.first() {
            Some(issue) => match issue.code.as_ref() {
                IssueType::Invalid(_) => axum::http::StatusCode::BAD_REQUEST,
                IssueType::NotFound(_) => axum::http::StatusCode::NOT_FOUND,
                IssueType::Forbidden(_) => axum::http::StatusCode::FORBIDDEN,
                IssueType::Conflict(_) => axum::http::StatusCode::CONFLICT,
                _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            },
            None => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status_code, response).into_response()
    }
}
