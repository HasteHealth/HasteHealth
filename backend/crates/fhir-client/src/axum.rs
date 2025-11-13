use axum::response::IntoResponse;
use http::{HeaderMap, StatusCode};
use oxidized_fhir_model::r4::generated::{
    resources::{Bundle, BundleEntry, Resource},
    terminology::{BundleType, IssueType},
    types::FHIRUnsignedInt,
};
use oxidized_fhir_operation_error::OperationOutcomeError;

use crate::request::FHIRResponse;

pub fn to_bundle(bundle_type: BundleType, total: Option<i64>, resources: Vec<Resource>) -> Bundle {
    Bundle {
        id: None,
        meta: None,
        total: total.map(|t| {
            Box::new(FHIRUnsignedInt {
                value: Some(t as u64),
                ..Default::default()
            })
        }),
        type_: Box::new(bundle_type),
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
        let mut header = HeaderMap::new();
        header.insert("Content-Type", "application/fhir+json".parse().unwrap());
        match self {
            FHIRResponse::Create(response) => (
                StatusCode::CREATED,
                header,
                // Unwrap should be safe here.
                oxidized_fhir_serialization_json::to_string(&response.resource).unwrap(),
            )
                .into_response(),
            FHIRResponse::Read(response) => {
                if let Some(resource) = response.resource {
                    (
                        StatusCode::OK,
                        header,
                        // Unwrap should be safe here.
                        oxidized_fhir_serialization_json::to_string(&resource).unwrap(),
                    )
                        .into_response()
                } else {
                    OperationOutcomeError::error(
                        IssueType::NotFound(None),
                        "Resource not found.".to_string(),
                    )
                    .into_response()
                }
            }
            FHIRResponse::VersionRead(response) => (
                StatusCode::OK,
                header,
                // Unwrap should be safe here.
                oxidized_fhir_serialization_json::to_string(&response.resource).unwrap(),
            )
                .into_response(),
            FHIRResponse::Update(response) => (
                StatusCode::OK,
                header,
                // Unwrap should be safe here.
                oxidized_fhir_serialization_json::to_string(&response.resource).unwrap(),
            )
                .into_response(),
            FHIRResponse::Capabilities(response) => (
                StatusCode::OK,
                header,
                // Unwrap should be safe here.
                oxidized_fhir_serialization_json::to_string(&response.capabilities).unwrap(),
            )
                .into_response(),
            FHIRResponse::HistoryInstance(response) => {
                let bundle = to_bundle(BundleType::History(None), None, response.resources);
                (
                    StatusCode::OK,
                    header,
                    // Unwrap should be safe here.
                    oxidized_fhir_serialization_json::to_string(&bundle).unwrap(),
                )
                    .into_response()
            }
            FHIRResponse::HistoryType(response) => {
                let bundle = to_bundle(BundleType::History(None), None, response.resources);
                (
                    StatusCode::OK,
                    header,
                    // Unwrap should be safe here.
                    oxidized_fhir_serialization_json::to_string(&bundle).unwrap(),
                )
                    .into_response()
            }
            FHIRResponse::HistorySystem(response) => {
                let bundle = to_bundle(BundleType::History(None), None, response.resources);
                (
                    StatusCode::OK,
                    header,
                    // Unwrap should be safe here.
                    oxidized_fhir_serialization_json::to_string(&bundle).unwrap(),
                )
                    .into_response()
            }
            FHIRResponse::SearchType(response) => {
                let bundle = to_bundle(
                    BundleType::Searchset(None),
                    response.total,
                    response.resources,
                );
                (
                    StatusCode::OK,
                    header,
                    // Unwrap should be safe here.
                    oxidized_fhir_serialization_json::to_string(&bundle).unwrap(),
                )
                    .into_response()
            }
            FHIRResponse::SearchSystem(response) => {
                let bundle = to_bundle(
                    BundleType::Searchset(None),
                    response.total,
                    response.resources,
                );
                (
                    StatusCode::OK,
                    header,
                    // Unwrap should be safe here.
                    oxidized_fhir_serialization_json::to_string(&bundle).unwrap(),
                )
                    .into_response()
            }
            FHIRResponse::Batch(response) => {
                (
                    StatusCode::OK,
                    header,
                    // Unwrap should be safe here.
                    oxidized_fhir_serialization_json::to_string(&response.resource).unwrap(),
                )
                    .into_response()
            }
            FHIRResponse::InvokeInstance(response) => {
                (
                    StatusCode::OK,
                    header,
                    // Unwrap should be safe here.
                    oxidized_fhir_serialization_json::to_string(&response.resource).unwrap(),
                )
                    .into_response()
            }
            FHIRResponse::InvokeType(response) => {
                (
                    StatusCode::OK,
                    header,
                    // Unwrap should be safe here.
                    oxidized_fhir_serialization_json::to_string(&response.resource).unwrap(),
                )
                    .into_response()
            }
            FHIRResponse::InvokeSystem(response) => {
                (
                    StatusCode::OK,
                    header,
                    // Unwrap should be safe here.
                    oxidized_fhir_serialization_json::to_string(&response.resource).unwrap(),
                )
                    .into_response()
            }
            FHIRResponse::DeleteInstance(_response) => {
                (StatusCode::NO_CONTENT, header, "").into_response()
            }
            FHIRResponse::Patch(fhirpatch_response) => (
                StatusCode::OK,
                header,
                // Unwrap should be safe here.
                oxidized_fhir_serialization_json::to_string(&fhirpatch_response.resource).unwrap(),
            )
                .into_response(),
            FHIRResponse::DeleteType(_fhirdelete_type_response) => {
                (StatusCode::NO_CONTENT, header, "").into_response()
            }
            FHIRResponse::DeleteSystem(_fhirdelete_system_response) => {
                (StatusCode::NO_CONTENT, header, "").into_response()
            }
            FHIRResponse::Transaction(fhirtransaction_response) => {
                (
                    StatusCode::OK,
                    header,
                    // Unwrap should be safe here.
                    oxidized_fhir_serialization_json::to_string(&fhirtransaction_response.resource)
                        .unwrap(),
                )
                    .into_response()
            }
        }
    }
}
