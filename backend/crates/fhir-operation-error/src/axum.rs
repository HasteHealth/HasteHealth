use std::error::Error;

use axum::response::IntoResponse;

use crate::OperationOutcomeError;

impl IntoResponse for OperationOutcomeError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("OperationOutcomeError source: {:?}", &self.source());
        let outcome = self.outcome;
        let response = oxidized_fhir_serialization_json::to_string(&outcome)
            .expect("Failed to serialize OperationOutcome");

        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, response).into_response()
    }
}
