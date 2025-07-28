use axum::response::IntoResponse;
use core::panic;
use http::StatusCode;

use crate::request::FHIRResponse;

impl IntoResponse for FHIRResponse {
    fn into_response(self) -> axum::response::Response {
        match &self {
            FHIRResponse::Create(response) => (
                StatusCode::CREATED,
                // Unwrap should be safe here.
                oxidized_fhir_serialization_json::to_string(&response.resource).unwrap(),
            )
                .into_response(),
            FHIRResponse::Read(response) => (
                StatusCode::OK,
                // Unwrap should be safe here.
                oxidized_fhir_serialization_json::to_string(&response.resource).unwrap(),
            )
                .into_response(),
            _ => panic!("Unsupported FHIRResponse type"),
        }
    }
}
