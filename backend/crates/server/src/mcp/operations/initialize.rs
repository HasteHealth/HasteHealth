use crate::{
    fhir_client::ServerCTX,
    mcp::{
        error::MCPError,
        request::InitializeRequest,
        schemas::schema_2025_11_25::{
            Implementation, InitializeResult, ServerCapabilities, ServerCapabilitiesTools,
        },
    },
};
use haste_fhir_client::FHIRClient;
use haste_fhir_operation_error::OperationOutcomeError;
use std::{collections::HashMap, sync::Arc};

pub async fn initialize<
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>(
    _ctx: Arc<ServerCTX<Client>>,
    _request: &InitializeRequest,
) -> Result<InitializeResult, MCPError<serde_json::Value>> {
    Ok(InitializeResult {
        capabilities: ServerCapabilities {
            completions: serde_json::Map::new(),
            experimental: HashMap::new(),
            logging: serde_json::Map::new(),
            prompts: None,
            resources: None,
            tasks: None,
            tools: Some(ServerCapabilitiesTools {
                list_changed: Some(false),
            }),
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
    })
}
