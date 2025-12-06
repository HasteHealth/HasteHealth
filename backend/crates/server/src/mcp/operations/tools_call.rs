use crate::{
    fhir_client::ServerCTX,
    mcp::{
        error::{MCPError, MCPErrorDetail},
        request::CallToolRequest,
        schemas::schema_2025_11_25::{CallToolResult, ContentBlock, TextContent, TextContentMeta},
    },
};
use haste_fhir_client::{FHIRClient, url::ParsedParameters};
use haste_fhir_model::r4::generated::{resources::ResourceType, terminology::IssueType};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::Repository;
use std::{collections::HashMap, sync::Arc};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(tag = "operation")]
enum Arguments {
    #[serde(rename = "search-type")]
    FHIRSearch {
        search_parameters: Option<HashMap<String, String>>,
    },
}

pub async fn tools_call<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    ctx: Arc<ServerCTX<Repo, Search, Terminology>>,
    request: CallToolRequest,
) -> Result<CallToolResult, MCPError<serde_json::Value>> {
    let arguments = serde_json::from_value::<Arguments>(
        request.params.arguments.unwrap_or_default(),
    )
    .map_err(|e| {
        OperationOutcomeError::error(
            IssueType::Invalid(None),
            format!("Failed to parse tool arguments: '{}'", e.to_string()),
        )
    })?;

    let content: String = match arguments {
        Arguments::FHIRSearch { search_parameters } => {
            let resource_type =
                ResourceType::try_from(request.params.name.as_str()).map_err(|_| MCPError {
                    id: request.id.clone(),
                    jsonrpc: "2.0".to_string(),
                    error: MCPErrorDetail {
                        code: 400,
                        message: "Invalid resource type provided in arguments".to_string(),
                        data: None,
                    },
                })?;

            let parsed_parameters = ParsedParameters::try_from(
                &search_parameters.unwrap_or_default(),
            )
            .map_err(|_| MCPError {
                id: request.id.clone(),
                jsonrpc: "2.0".to_string(),
                error: MCPErrorDetail {
                    code: 400,
                    message: "Failed to parse search parameters".to_string(),
                    data: None,
                },
            })?;

            let result = ctx
                .client
                .search_type(ctx.clone(), resource_type, parsed_parameters)
                .await?;

            let result = haste_fhir_serialization_json::to_string(&result).map_err(|e| {
                OperationOutcomeError::error(
                    IssueType::Processing(None),
                    format!("Failed to serialize search result: '{}'", e.to_string()),
                )
            })?;

            result
        }
    };

    Ok(CallToolResult {
        structured_content: Some(serde_json::from_str::<serde_json::Value>(&content).map_err(
            |e| {
                OperationOutcomeError::error(
                    IssueType::Processing(None),
                    format!("Failed to parse search result JSON: '{}'", e.to_string()),
                )
            },
        )?),
        content: vec![ContentBlock::TextContent(TextContent {
            annotations: None,
            meta: Some(TextContentMeta {}),
            text: content,
            type_: "text".to_string(),
        })],
        is_error: Some(false),
        meta: None,
    })
}
