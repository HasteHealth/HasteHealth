use crate::{
    fhir_client::ServerCTX,
    mcp::{
        error::MCPError,
        request::InitializeRequest,
        schemas::types::{
            Implementation, InitializeResult, ServerCapabilities, ServerCapabilitiesTools,
        },
    },
};
use haste_fhir_client::FHIRClient;
use haste_fhir_operation_error::OperationOutcomeError;
use std::sync::Arc;

pub async fn initialize<
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>(
    _ctx: Arc<ServerCTX<Client>>,
    _request: &InitializeRequest,
) -> Result<InitializeResult, MCPError<serde_json::Value>> {
    Ok(InitializeResult {
        capabilities: ServerCapabilities {
            tools: Some(ServerCapabilitiesTools {
                list_changed: Some(false),
            }),
            ..ServerCapabilities::default()
        },
        instructions: None,
        meta: None,
        protocol_version: "2025-03-26".to_string(),
        server_info: Implementation {
            name: "Haste Health MCP Server".to_string(),
            version: "0.0.1".to_string(),
            title: Some("Haste Health MCP Server".to_string()),
        },
    })
}
