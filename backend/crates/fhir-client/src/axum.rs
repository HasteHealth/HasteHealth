use crate::request::{
    DeleteResponse, FHIRResponse, HistoryResponse, InvokeResponse, SearchResponse,
};
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
            axum::http::header::LAST_MODIFIED,
            last_modified
                .format("%a, %d %b %G %H:%M:%S GMT")
                .parse()
                .unwrap(),
        );
    }
    if let Some(version_id) = version_id {
        headers.insert(
            axum::http::header::ETAG,
            format!("W/\"{}\"", version_id).parse().unwrap(),
        );
    }
}

fn add_headers(response: &FHIRResponse) -> HeaderMap {
    let mut header = HeaderMap::new();
    header.insert(
        axum::http::header::CONTENT_TYPE,
        "application/fhir+json".parse().unwrap(),
    );

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
            FHIRResponse::History(history_response) => match history_response {
                HistoryResponse::Instance(response) => (
                    StatusCode::OK,
                    header,
                    // Unwrap should be safe here.
                    haste_fhir_serialization_json::to_string(&response.bundle).unwrap(),
                )
                    .into_response(),
                HistoryResponse::Type(response) => (
                    StatusCode::OK,
                    header,
                    // Unwrap should be safe here.
                    haste_fhir_serialization_json::to_string(&response.bundle).unwrap(),
                )
                    .into_response(),
                HistoryResponse::System(response) => (
                    StatusCode::OK,
                    header,
                    // Unwrap should be safe here.
                    haste_fhir_serialization_json::to_string(&response.bundle).unwrap(),
                )
                    .into_response(),
            },
            FHIRResponse::Search(search_response) => match search_response {
                SearchResponse::Type(response) => (
                    StatusCode::OK,
                    header,
                    // Unwrap should be safe here.
                    haste_fhir_serialization_json::to_string(&response.bundle).unwrap(),
                )
                    .into_response(),
                SearchResponse::System(response) => (
                    StatusCode::OK,
                    header,
                    // Unwrap should be safe here.
                    haste_fhir_serialization_json::to_string(&response.bundle).unwrap(),
                )
                    .into_response(),
            },
            FHIRResponse::Batch(response) => {
                (
                    StatusCode::OK,
                    header,
                    // Unwrap should be safe here.
                    haste_fhir_serialization_json::to_string(&response.resource).unwrap(),
                )
                    .into_response()
            }
            FHIRResponse::Invoke(invoke_response) => match invoke_response {
                InvokeResponse::Instance(invoke_response) => {
                    (
                        StatusCode::OK,
                        header,
                        // Unwrap should be safe here.
                        haste_fhir_serialization_json::to_string(&invoke_response.resource)
                            .unwrap(),
                    )
                        .into_response()
                }
                InvokeResponse::Type(invoke_response) => {
                    (
                        StatusCode::OK,
                        header,
                        // Unwrap should be safe here.
                        haste_fhir_serialization_json::to_string(&invoke_response.resource)
                            .unwrap(),
                    )
                        .into_response()
                }
                InvokeResponse::System(invoke_response) => {
                    (
                        StatusCode::OK,
                        header,
                        // Unwrap should be safe here.
                        haste_fhir_serialization_json::to_string(&invoke_response.resource)
                            .unwrap(),
                    )
                        .into_response()
                }
            },

            FHIRResponse::Delete(delete_response) => match delete_response {
                DeleteResponse::Instance(_) => (StatusCode::NO_CONTENT, header, "").into_response(),
                DeleteResponse::Type(_) => (StatusCode::NO_CONTENT, header, "").into_response(),
                DeleteResponse::System(_) => (StatusCode::NO_CONTENT, header, "").into_response(),
            },

            FHIRResponse::Patch(fhirpatch_response) => (
                StatusCode::OK,
                header,
                // Unwrap should be safe here.
                haste_fhir_serialization_json::to_string(&fhirpatch_response.resource).unwrap(),
            )
                .into_response(),

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
