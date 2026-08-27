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

fn add_resource_headers(headers: &mut HeaderMap, resource: &Resource) {
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
            format!("W/\"{version_id}\"").parse().unwrap(),
        );
    }
}

fn add_headers(response: &FHIRResponse) -> HeaderMap {
    let mut header = HeaderMap::new();
    header.insert(
        axum::http::header::CONTENT_TYPE,
        "application/fhir+json; charset=utf-8".parse().unwrap(),
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
    }

    header
}

impl IntoResponse for FHIRResponse {
    fn into_response(self) -> axum::response::Response {
        let header = add_headers(&self);

        match self {
            FHIRResponse::Create(response) => {
                FHIRResponse::created_json(header, &response.resource)
            }

            FHIRResponse::Read(response) => match response.resource {
                Some(resource) => FHIRResponse::ok_json(header, &resource),
                None => FHIRResponse::not_found(),
            },

            FHIRResponse::VersionRead(response) => {
                FHIRResponse::ok_json(header, &response.resource)
            }

            FHIRResponse::Update(response) => FHIRResponse::ok_json(header, &response.resource),

            FHIRResponse::Capabilities(response) => {
                FHIRResponse::ok_json(header, &response.capabilities)
            }

            FHIRResponse::History(history_response) => {
                FHIRResponse::history_response(header, history_response)
            }

            FHIRResponse::Search(search_response) => {
                FHIRResponse::search_response(header, search_response)
            }

            FHIRResponse::Batch(response) => FHIRResponse::ok_json(header, &response.resource),

            FHIRResponse::Invoke(invoke_response) => {
                FHIRResponse::invoke_response(header, invoke_response)
            }

            FHIRResponse::Delete(delete_response) => {
                FHIRResponse::delete_response(header, delete_response)
            }

            FHIRResponse::Patch(response) => FHIRResponse::ok_json(header, &response.resource),

            FHIRResponse::Transaction(response) => {
                FHIRResponse::ok_json(header, &response.resource)
            }
        }
    }
}

impl FHIRResponse {
    fn json_response<T: serde::Serialize>(
        status: StatusCode,
        header: HeaderMap,
        value: &T,
    ) -> axum::response::Response {
        (
            status,
            header,
            // Unwrap should be safe here.
            serde_json::to_string(value).unwrap(),
        )
            .into_response()
    }

    fn ok_json<T: serde::Serialize>(header: HeaderMap, value: &T) -> axum::response::Response {
        Self::json_response(StatusCode::OK, header, value)
    }

    fn created_json<T: serde::Serialize>(header: HeaderMap, value: &T) -> axum::response::Response {
        Self::json_response(StatusCode::CREATED, header, value)
    }

    fn no_content(header: HeaderMap) -> axum::response::Response {
        (StatusCode::NO_CONTENT, header, "").into_response()
    }

    fn not_found() -> axum::response::Response {
        OperationOutcomeError::error(IssueType::not_found(), "Resource not found.".to_string())
            .into_response()
    }

    fn history_response(header: HeaderMap, response: HistoryResponse) -> axum::response::Response {
        match response {
            HistoryResponse::Instance(response) => Self::ok_json(header, &response.bundle),
            HistoryResponse::Type(response) => Self::ok_json(header, &response.bundle),
            HistoryResponse::System(response) => Self::ok_json(header, &response.bundle),
        }
    }

    fn search_response(header: HeaderMap, response: SearchResponse) -> axum::response::Response {
        match response {
            SearchResponse::Type(response) => Self::ok_json(header, &response.bundle),
            SearchResponse::System(response) => Self::ok_json(header, &response.bundle),
        }
    }

    fn invoke_response(header: HeaderMap, response: InvokeResponse) -> axum::response::Response {
        match response {
            InvokeResponse::Instance(response) => Self::ok_json(header, &response.resource),
            InvokeResponse::Type(response) => Self::ok_json(header, &response.resource),
            InvokeResponse::System(response) => Self::ok_json(header, &response.resource),
        }
    }

    fn delete_response(header: HeaderMap, response: DeleteResponse) -> axum::response::Response {
        match response {
            DeleteResponse::Instance(response) => Self::ok_json(header, &response.resource),
            DeleteResponse::Type(_) | DeleteResponse::System(_) => Self::no_content(header),
        }
    }
}
