use axum::response::IntoResponse;
use core::panic;
use http::StatusCode;
use oxidized_fhir_model::r4::types::{Bundle, BundleEntry, FHIRCode, Resource};

use crate::request::FHIRResponse;

fn to_bundle(bundle_type: String, resources: Vec<Resource>) -> Bundle {
    Bundle {
        id: None,
        meta: None,
        type_: Box::new(FHIRCode {
            value: Some(bundle_type),
            ..Default::default()
        }),
        entry: Some(
            resources
                .into_iter()
                .map(|r| BundleEntry {
                    resource: Some(Box::new(r)),
                    ..Default::default()
                })
                .collect(),
        ),
        ..Default::default()
    }
}

impl IntoResponse for FHIRResponse {
    fn into_response(self) -> axum::response::Response {
        match self {
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
            FHIRResponse::VersionRead(response) => (
                StatusCode::OK,
                // Unwrap should be safe here.
                oxidized_fhir_serialization_json::to_string(&response.resource).unwrap(),
            )
                .into_response(),
            FHIRResponse::Update(response) => (
                StatusCode::OK,
                // Unwrap should be safe here.
                oxidized_fhir_serialization_json::to_string(&response.resource).unwrap(),
            )
                .into_response(),
            FHIRResponse::HistoryInstance(response) => {
                let bundle = to_bundle("history".to_string(), response.resources);
                (
                    StatusCode::OK,
                    // Unwrap should be safe here.
                    oxidized_fhir_serialization_json::to_string(&bundle).unwrap(),
                )
                    .into_response()
            }
            FHIRResponse::HistoryType(response) => {
                let bundle = to_bundle("history".to_string(), response.resources);
                (
                    StatusCode::OK,
                    // Unwrap should be safe here.
                    oxidized_fhir_serialization_json::to_string(&bundle).unwrap(),
                )
                    .into_response()
            }
            FHIRResponse::HistorySystem(response) => {
                let bundle = to_bundle("history".to_string(), response.resources);
                (
                    StatusCode::OK,
                    // Unwrap should be safe here.
                    oxidized_fhir_serialization_json::to_string(&bundle).unwrap(),
                )
                    .into_response()
            }
            FHIRResponse::SearchType(response) => {
                let bundle = to_bundle("searchset".to_string(), response.resources);
                (
                    StatusCode::OK,
                    // Unwrap should be safe here.
                    oxidized_fhir_serialization_json::to_string(&bundle).unwrap(),
                )
                    .into_response()
            }
            _ => panic!("Unsupported FHIRResponse type"),
        }
    }
}
