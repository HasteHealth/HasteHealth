use std::{collections::HashMap, sync::Arc};

use crate::{
    fhir_client::ServerCTX,
    mcp::{
        error::MCPError,
        schemas::schema_2025_11_25::{ListToolsRequest, ListToolsResult, Tool, ToolInputSchema},
    },
};
use haste_fhir_client::FHIRClient;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::Repository;

pub async fn list_tools<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    ctx: Arc<ServerCTX<Repo, Search, Terminology>>,
    _request: &ListToolsRequest,
) -> Result<ListToolsResult, MCPError<serde_json::Value>> {
    let _capabilities = ctx.client.capabilities(ctx.clone()).await?;

    Ok(ListToolsResult {
        tools: vec![Tool {
            annotations: None,
            description: None,
            execution: None,
            icons: vec![],
            input_schema: ToolInputSchema {
                properties: HashMap::new(),
                required: vec![],
                schema: None,
                type_: "object".to_string(),
            },
            meta: None,
            name: "Tool".to_string(),
            output_schema: None,
            title: Some("Tool".to_string()),
        }],
        meta: None,
        next_cursor: None,
    })
}
