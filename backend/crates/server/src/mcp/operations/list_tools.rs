use crate::{
    fhir_client::ServerCTX,
    mcp::{
        error::MCPError,
        request::ListToolsRequest,
        schemas::schema_2025_11_25::{ListToolsResult, Tool, ToolInputSchema},
    },
};
use haste_fhir_client::FHIRClient;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::Repository;
use std::{collections::HashMap, sync::Arc};

pub async fn list_tools<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    ctx: Arc<ServerCTX<Repo, Search, Terminology>>,
    _request: &ListToolsRequest,
) -> Result<ListToolsResult, MCPError<serde_json::Value>> {
    let capabilities = ctx.client.capabilities(ctx.clone()).await?;

    let resource_tools = capabilities
        .rest
        .unwrap_or_default()
        .into_iter()
        .map(|r| r.resource.unwrap_or_default())
        .flatten()
        .map(|resource| {
            let type_: Option<String> = resource.type_.as_ref().into();

            Tool {
                annotations: None,
                description: resource.profile.and_then(|p| p.value),
                execution: None,
                icons: vec![],
                input_schema: ToolInputSchema {
                    properties: HashMap::new(),
                    required: vec![],
                    schema: None,
                    type_: "object".to_string(),
                },
                meta: None,
                name: type_.clone().unwrap_or("Unknown".to_string()),
                output_schema: None,
                title: type_,
            }
        });

    Ok(ListToolsResult {
        tools: resource_tools.collect(),
        meta: None,
        next_cursor: None,
    })
}
