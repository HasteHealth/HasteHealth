use crate::{
    mcp::schemas::schema_2025_11_25::{
        ClientNotification, ClientRequest, Implementation, InitializeResult, RequestId,
        ServerCapabilities, ServerResult,
    },
    services::AppState,
};
use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::Repository;
use reqwest::Method;
use std::{collections::HashMap, sync::Arc};

#[derive(serde::Serialize)]
pub struct JSONRPCResult<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<RequestId>,
    jsonrpc: String,
    result: T,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum MCPRequest {
    ClientRequest(ClientRequest),
    ClientNotification(ClientNotification),
}

pub async fn mcp_handler<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    method: Method,
    State(state): State<Arc<AppState<Repo, Search, Terminology>>>,
    Json(mcp_request): Json<MCPRequest>,
) -> Result<Response, OperationOutcomeError> {
    match mcp_request {
        MCPRequest::ClientRequest(ClientRequest::InitializeRequest(initialize_request)) => {
            let result = ServerResult {
                subtype_1: Some(InitializeResult {
                    capabilities: ServerCapabilities {
                        completions: serde_json::Map::new(),
                        experimental: HashMap::new(),
                        logging: serde_json::Map::new(),
                        prompts: None,
                        resources: None,
                        tasks: None,
                        tools: None,
                    },
                    instructions: None,
                    meta: None,
                    protocol_version: "2025-03-26".to_string(),
                    server_info: Implementation {
                        description: None,
                        icons: vec![],
                        name: "Haste Health MCP Server".to_string(),
                        title: Some("Haste Health MCP Server".to_string()),
                        version: "0.0.1".to_string(),
                        website_url: Some("https://haste.health".to_string()),
                    },
                }),
                ..ServerResult::default()
            };
            Ok(Json(JSONRPCResult {
                id: Some(initialize_request.id),
                result,
                jsonrpc: "2.0".to_string(),
            })
            .into_response())
        }
        MCPRequest::ClientNotification(ClientNotification::InitializedNotification(
            _notification,
        )) => Ok(StatusCode::OK.into_response()),
        _ => Err(OperationOutcomeError::error(
            IssueType::NotSupported(None),
            "Only InitializeRequest is implemented".to_string(),
        )),
    }
}
