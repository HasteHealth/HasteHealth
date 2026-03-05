use crate::{
    fhir_client::ServerCTX,
    mcp::{
        error::{MCPError, MCPErrorDetail},
        operations::{
            GET_SEARCH_PARAMETERS_TOOL_NAME, R4_SEARCH_TOOL_NAME, search_tool_parameters,
        },
        request::CallToolRequest,
        schemas::schema_2025_11_25::{CallToolResult, ContentBlock, TextContent, TextContentMeta},
    },
};
use haste_fhir_client::{FHIRClient, url::ParsedParameters};
use haste_fhir_model::r4::generated::{resources::ResourceType, terminology::IssueType};
use haste_fhir_operation_error::OperationOutcomeError;
use std::{collections::HashMap, sync::Arc};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct FHIRSearchArguments {
    #[serde(rename = "resourceType")]
    resource_type: String,
    search_parameters: Option<HashMap<String, String>>,
}

pub async fn tools_call<
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>(
    ctx: Arc<ServerCTX<Client>>,
    request: CallToolRequest,
) -> Result<CallToolResult, MCPError<serde_json::Value>> {
    match request.params.name.as_str() {
        R4_SEARCH_TOOL_NAME => {
            let FHIRSearchArguments {
                resource_type,
                search_parameters,
            } = serde_json::from_value::<FHIRSearchArguments>(
                request.params.arguments.unwrap_or_default(),
            )
            .map_err(|e| {
                OperationOutcomeError::error(
                    IssueType::Invalid(None),
                    format!("Failed to parse tool arguments: '{}'", e.to_string()),
                )
            })?;

            let resource_type =
                ResourceType::try_from(resource_type.as_str()).map_err(|_| MCPError {
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

            Ok(CallToolResult {
                structured_content: Some(
                    serde_json::from_str::<serde_json::Value>(&result).map_err(|e| {
                        OperationOutcomeError::error(
                            IssueType::Processing(None),
                            format!("Failed to parse search result JSON: '{}'", e.to_string()),
                        )
                    })?,
                ),
                content: vec![ContentBlock::TextContent(TextContent {
                    annotations: None,
                    meta: Some(TextContentMeta {}),
                    text: result,
                    type_: "text".to_string(),
                })],
                is_error: Some(false),
                meta: None,
            })
        }

        GET_SEARCH_PARAMETERS_TOOL_NAME => {
            let capabilities = ctx.client.capabilities(ctx.clone()).await?;
            let resource_capability_statment = capabilities
                .rest
                .unwrap_or_default()
                .into_iter()
                .filter_map(|rest| rest.resource)
                .flatten()
                .find(|r| {
                    let rc_type: Option<String> = r.type_.as_ref().into();
                    rc_type.unwrap_or_default()
                        == request
                            .params
                            .arguments
                            .as_ref()
                            .and_then(|args| args.get("resourceType"))
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                });

            let Some(resource_capability_statment_params) = resource_capability_statment
                .as_ref()
                .and_then(|rc| rc.searchParam.as_ref())
            else {
                return Err(MCPError {
                    id: request.id.clone(),
                    jsonrpc: "2.0".to_string(),
                    error: MCPErrorDetail {
                        code: 400,
                        message: "Invalid resourceType could not find search parameters"
                            .to_string(),
                        data: None,
                    },
                });
            };

            let parameters = search_tool_parameters(resource_capability_statment_params);

            Ok(CallToolResult {
                content: vec![ContentBlock::TextContent(TextContent {
                    annotations: None,
                    meta: Some(TextContentMeta {}),
                    text: serde_json::to_string(&parameters).unwrap_or_default(),
                    type_: "text".to_string(),
                })],
                structured_content: Some(parameters),
                is_error: Some(false),
                meta: None,
            })
        }
        _ => Err(MCPError {
            id: request.id.clone(),
            jsonrpc: "2.0".to_string(),
            error: MCPErrorDetail {
                code: 400,
                message: "Unknown tool name".to_string(),
                data: None,
            },
        }),
    }
}
