use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::Repository;
use std::{collections::HashMap, sync::Arc};

use crate::{
    fhir_client::ServerCTX,
    mcp::{
        error::MCPError,
        schemas::schema_2025_11_25::{
            Implementation, InitializeRequest, InitializeResult, ServerCapabilities,
            ServerCapabilitiesTools,
        },
    },
};

pub async fn initialize<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    _ctx: Arc<ServerCTX<Repo, Search, Terminology>>,
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
