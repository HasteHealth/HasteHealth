use axum::response::IntoResponse;

use crate::OperationError;

impl IntoResponse for OperationError {
    fn into_response(self) -> axum::response::Response {
        let outcome = self.outcome;
        let response = fhir_serialization_json::to_string(&outcome)
            .expect("Failed to serialize OperationOutcome");
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, response).into_response()
    }
}
