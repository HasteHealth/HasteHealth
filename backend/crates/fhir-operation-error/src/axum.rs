use std::error::Error;

use axum::response::IntoResponse;

use crate::{OperationOutcomeCodes, OperationOutcomeError};

impl IntoResponse for OperationOutcomeError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("OperationOutcomeError source: {:?}", &self.source());
        let outcome = self.outcome;
        let response = oxidized_fhir_serialization_json::to_string(&outcome)
            .expect("Failed to serialize OperationOutcome");

        let status_code = match outcome.issue.first() {
            Some(issue) => match issue.code.value.as_ref().map(|c| c.clone().try_into()) {
                Some(Ok(OperationOutcomeCodes::Invalid)) => axum::http::StatusCode::BAD_REQUEST,
                Some(Ok(OperationOutcomeCodes::NotFound)) => axum::http::StatusCode::NOT_FOUND,
                Some(Ok(OperationOutcomeCodes::Forbidden)) => axum::http::StatusCode::FORBIDDEN,
                Some(Ok(OperationOutcomeCodes::Conflict)) => axum::http::StatusCode::CONFLICT,

                _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            },
            None => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status_code, response).into_response()
    }
}
