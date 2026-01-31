use crate::{
    load_artifacts::{get_all_sds, get_all_sps},
    services::AppState,
};
use axum::{extract::State, http::HeaderMap, response::IntoResponse, response::Response};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_openapi_schema_generator::OpenAPI;
use haste_repository::Repository;
use reqwest::header::CONTENT_TYPE;
use std::sync::{Arc, LazyLock};
use tokio::sync::Mutex;

static OPENAPI_DOCUMENT: LazyLock<Mutex<Option<OpenAPI>>> = LazyLock::new(|| Mutex::new(None));

pub async fn openapi_document_handler<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    State(state): State<Arc<AppState<Repo, Search, Terminology>>>,
) -> Result<Response, OperationOutcomeError> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());

    if let Some(doc) = &*OPENAPI_DOCUMENT.lock().await {
        Ok((
            headers,
            serde_json::to_string(doc).map_err(|_e| {
                OperationOutcomeError::error(
                    IssueType::Exception(None),
                    "Failed to serialize OpenAPI document".to_string(),
                )
            })?,
        )
            .into_response())
    } else {
        let sps = get_all_sps(state.repo.as_ref(), state.search.as_ref()).await?;
        let sds = get_all_sds(
            &["resource", "complex-type"],
            state.repo.as_ref(),
            state.search.as_ref(),
        )
        .await?;

        let api_url = state
            .config
            .get(crate::ServerEnvironmentVariables::APIURI)?;

        let api_version = env!("CARGO_PKG_VERSION");

        let openapi_document = haste_openapi_schema_generator::open_api_schema_generator(
            &api_url,
            api_version,
            &sds,
            &sps,
        )?;

        let mut doc_lock = OPENAPI_DOCUMENT.lock().await;

        let response = Ok((
            headers,
            serde_json::to_string(&openapi_document).map_err(|_e| {
                OperationOutcomeError::error(
                    IssueType::Exception(None),
                    "Failed to serialize OpenAPI document".to_string(),
                )
            })?,
        )
            .into_response());

        *doc_lock = Some(openapi_document);

        response
    }
}
