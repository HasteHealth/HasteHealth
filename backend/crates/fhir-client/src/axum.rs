use crate::request::FHIRResponse;
use axum::response::IntoResponse;
use haste_fhir_model::r4::generated::{
    resources::Resource,
    terminology::IssueType,
    types::{FHIRId, FHIRInstant},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_reflect::MetaValue;
use http::{HeaderMap, StatusCode};

fn add_resource_headers(headers: &mut HeaderMap, resource: &Resource) -> () {
    let _id = resource
        .get_field("id")
        .and_then(|id| id.as_any().downcast_ref::<String>());

    let meta = resource.get_field("meta");

    let last_modified = meta
        .and_then(|meta| meta.get_field("lastUpdated"))
        .and_then(|lu| lu.as_any().downcast_ref::<Box<FHIRInstant>>())
        .and_then(|lu| lu.value.as_ref());

    let version_id = meta
        .and_then(|meta| meta.get_field("versionId"))
        .and_then(|vid| vid.as_any().downcast_ref::<Box<FHIRId>>())
        .and_then(|vid| vid.value.as_ref());

    if let Some(last_modified) = last_modified {
        headers.insert(
            "Last-Modified",
            last_modified
                .format("%a, %d %b %G %H:%M:%S GMT")
                .parse()
                .unwrap(),
        );
    }
    if let Some(version_id) = version_id {
        headers.insert("ETag", format!("W/\"{}\"", version_id).parse().unwrap());
    }
}

fn add_headers(response: &FHIRResponse) -> HeaderMap {
    let mut header = HeaderMap::new();
    header.insert("Content-Type", "application/fhir+json".parse().unwrap());

    match response {
        FHIRResponse::Create(resp) => {
            add_resource_headers(&mut header, &resp.resource);
        }
        FHIRResponse::Read(resp) => {
            if let Some(resource) = &resp.resource {
                add_resource_headers(&mut header, resource);
            }
        }
        FHIRResponse::VersionRead(resp) => {
            add_resource_headers(&mut header, &resp.resource);
        }
        FHIRResponse::Update(resp) => {
            add_resource_headers(&mut header, &resp.resource);
        }
        FHIRResponse::Patch(fhirpatch_response) => {
            add_resource_headers(&mut header, &fhirpatch_response.resource);
        }
        _ => {}
    };

    header
}

impl IntoResponse for FHIRResponse {
    fn into_response(self) -> axum::response::Response {
        let header = add_headers(&self);

        match self {
            FHIRResponse::Create(response) => (
                StatusCode::CREATED,
                header,
                // Unwrap should be safe here.
                haste_fhir_serialization_json::to_string(&response.resource).unwrap(),
            )
                .into_response(),
            FHIRResponse::Read(response) => {
                if let Some(resource) = response.resource {
                    (
                        StatusCode::OK,
                        header,
                        // Unwrap should be safe here.
                        haste_fhir_serialization_json::to_string(&resource).unwrap(),
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
                haste_fhir_serialization_json::to_string(&response.resource).unwrap(),
            )
                .into_response(),
            FHIRResponse::Update(response) => (
                StatusCode::OK,
                header,
                // Unwrap should be safe here.
                haste_fhir_serialization_json::to_string(&response.resource).unwrap(),
            )
                .into_response(),
            FHIRResponse::Capabilities(response) => (
                StatusCode::OK,
                header,
                // Unwrap should be safe here.
                haste_fhir_serialization_json::to_string(&response.capabilities).unwrap(),
            )
                .into_response(),
            FHIRResponse::HistoryInstance(response) => {
                (
                    StatusCode::OK,
                    header,
                    // Unwrap should be safe here.
                    haste_fhir_serialization_json::to_string(&response.bundle).unwrap(),
                )
                    .into_response()
            }
            FHIRResponse::HistoryType(response) => {
                (
                    StatusCode::OK,
                    header,
                    // Unwrap should be safe here.
                    haste_fhir_serialization_json::to_string(&response.bundle).unwrap(),
                )
                    .into_response()
            }
            FHIRResponse::HistorySystem(response) => {
                (
                    StatusCode::OK,
                    header,
                    // Unwrap should be safe here.
                    haste_fhir_serialization_json::to_string(&response.bundle).unwrap(),
                )
                    .into_response()
            }
            FHIRResponse::SearchType(response) => {
                (
                    StatusCode::OK,
                    header,
                    // Unwrap should be safe here.
                    haste_fhir_serialization_json::to_string(&response.bundle).unwrap(),
                )
                    .into_response()
            }
            FHIRResponse::SearchSystem(response) => {
                (
                    StatusCode::OK,
                    header,
                    // Unwrap should be safe here.
                    haste_fhir_serialization_json::to_string(&response.bundle).unwrap(),
                )
                    .into_response()
            }
            FHIRResponse::Batch(response) => {
                (
                    StatusCode::OK,
                    header,
                    // Unwrap should be safe here.
                    haste_fhir_serialization_json::to_string(&response.resource).unwrap(),
                )
                    .into_response()
            }
            FHIRResponse::InvokeInstance(response) => {
                (
                    StatusCode::OK,
                    header,
                    // Unwrap should be safe here.
                    haste_fhir_serialization_json::to_string(&response.resource).unwrap(),
                )
                    .into_response()
            }
            FHIRResponse::InvokeType(response) => {
                (
                    StatusCode::OK,
                    header,
                    // Unwrap should be safe here.
                    haste_fhir_serialization_json::to_string(&response.resource).unwrap(),
                )
                    .into_response()
            }
            FHIRResponse::InvokeSystem(response) => {
                (
                    StatusCode::OK,
                    header,
                    // Unwrap should be safe here.
                    haste_fhir_serialization_json::to_string(&response.resource).unwrap(),
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
                haste_fhir_serialization_json::to_string(&fhirpatch_response.resource).unwrap(),
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
                    haste_fhir_serialization_json::to_string(&fhirtransaction_response.resource)
                        .unwrap(),
                )
                    .into_response()
            }
        }
    }
}
