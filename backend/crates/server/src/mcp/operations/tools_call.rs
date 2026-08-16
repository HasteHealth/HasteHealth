use crate::{
    fhir_client::ServerCTX,
    mcp::{
        error::{MCPError, MCPErrorDetail},
        operations::{
            GET_SEARCH_PARAMETERS_TOOL_NAME, R4_BATCH_TOOL_NAME, R4_CAPABILITIES_TOOL_NAME,
            R4_CREATE_TOOL_NAME, R4_DELETE_TOOL_NAME, R4_HISTORY_INSTANCE_TOOL_NAME,
            R4_HISTORY_TYPE_TOOL_NAME, R4_PATCH_TOOL_NAME, R4_READ_TOOL_NAME, R4_SEARCH_TOOL_NAME,
            R4_TRANSACTION_TOOL_NAME, R4_UPDATE_TOOL_NAME, R4_VREAD_TOOL_NAME,
            search_tool_parameters,
        },
        request::CallToolRequest,
        schemas::types::{CallToolResult, ContentBlock, TextContent},
    },
};
use haste_fhir_client::{FHIRClient, url::ParsedParameters};
use haste_fhir_model::r4::generated::{
    resources::{Bundle, Resource, ResourceType},
    terminology::IssueType,
};
use haste_fhir_operation_error::OperationOutcomeError;
use json_patch::Patch;
use std::{collections::HashMap, sync::Arc};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct FHIRSearchArguments {
    #[serde(rename = "resourceType")]
    resource_type: String,
    search_parameters: Option<HashMap<String, String>>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct FHIRReadArguments {
    #[serde(rename = "resourceType")]
    resource_type: String,
    id: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct FHIRVReadArguments {
    #[serde(rename = "resourceType")]
    resource_type: String,
    id: String,
    #[serde(rename = "versionId")]
    version_id: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct FHIRCreateArguments {
    #[serde(rename = "resourceType")]
    resource_type: String,
    resource: serde_json::Value,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct FHIRUpdateArguments {
    #[serde(rename = "resourceType")]
    resource_type: String,
    id: String,
    resource: serde_json::Value,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct FHIRPatchArguments {
    #[serde(rename = "resourceType")]
    resource_type: String,
    id: String,
    patches: serde_json::Value,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct FHIRDeleteArguments {
    #[serde(rename = "resourceType")]
    resource_type: String,
    id: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct FHIRHistoryInstanceArguments {
    #[serde(rename = "resourceType")]
    resource_type: String,
    id: String,
    #[serde(rename = "_count")]
    count: Option<String>,
    #[serde(rename = "_since")]
    since: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct FHIRHistoryTypeArguments {
    #[serde(rename = "resourceType")]
    resource_type: String,
    #[serde(rename = "_count")]
    count: Option<String>,
    #[serde(rename = "_since")]
    since: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct FHIRBundleArguments {
    bundle: serde_json::Value,
}

/// Helper: parse resource type string into `ResourceType`, returning MCP error on failure.
fn parse_resource_type(
    resource_type: &str,
    request_id: &Option<crate::mcp::schemas::types::RequestId>,
) -> Result<ResourceType, MCPError<serde_json::Value>> {
    ResourceType::try_from(resource_type).map_err(|_| MCPError {
        id: request_id.clone(),
        jsonrpc: "2.0".to_string(),
        error: MCPErrorDetail {
            code: 400,
            message: format!("Invalid resource type: '{}'", resource_type),
            data: None,
        },
    })
}

/// Helper: parse tool arguments from JSON value.
fn parse_arguments<T: serde::de::DeserializeOwned>(
    arguments: Option<serde_json::Value>,
) -> Result<T, MCPError<serde_json::Value>> {
    serde_json::from_value::<T>(arguments.unwrap_or_default()).map_err(|e| {
        OperationOutcomeError::error(
            IssueType::invalid(),
            format!("Failed to parse tool arguments: '{}'", e),
        )
        .into()
    })
}

/// Helper: build a successful CallToolResult with text + structured content.
fn success_result(
    value: &serde_json::Value,
) -> Result<CallToolResult, MCPError<serde_json::Value>> {
    let text = serde_json::to_string(value).map_err(|e| {
        OperationOutcomeError::error(
            IssueType::processing(),
            format!("Failed to serialize result: '{}'", e),
        )
    })?;

    Ok(CallToolResult {
        structured_content: Some(value.clone()),
        content: vec![ContentBlock::Text(TextContent::new(text))],
        is_error: Some(false),
        meta: None,
    })
}

/// Helper: serialize a Resource to JSON value.
fn resource_to_json(resource: &Resource) -> Result<serde_json::Value, MCPError<serde_json::Value>> {
    serde_json::to_value(resource).map_err(|e| {
        OperationOutcomeError::error(
            IssueType::processing(),
            format!("Failed to serialize resource: '{}'", e),
        )
        .into()
    })
}

/// Helper: deserialize a JSON value into a FHIR Resource.
fn json_to_resource(value: serde_json::Value) -> Result<Resource, MCPError<serde_json::Value>> {
    serde_json::from_value::<Resource>(value).map_err(|e| {
        OperationOutcomeError::error(
            IssueType::invalid(),
            format!("Failed to parse FHIR resource: '{}'", e),
        )
        .into()
    })
}

/// Helper: convert history parameters into ParsedParameters.
fn history_params(
    count: Option<String>,
    since: Option<String>,
) -> Result<ParsedParameters, MCPError<serde_json::Value>> {
    let mut params = HashMap::new();
    if let Some(c) = count {
        params.insert("_count".to_string(), c);
    }
    if let Some(s) = since {
        params.insert("_since".to_string(), s);
    }
    ParsedParameters::try_from(&params).map_err(|_| {
        OperationOutcomeError::error(
            IssueType::invalid(),
            "Failed to parse history parameters".to_string(),
        )
        .into()
    })
}

pub async fn tools_call<
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>(
    ctx: Arc<ServerCTX<Client>>,
    request: CallToolRequest,
) -> Result<CallToolResult, MCPError<serde_json::Value>> {
    match request.params.name.as_str() {
        R4_SEARCH_TOOL_NAME => {
            let args: FHIRSearchArguments = parse_arguments(request.params.arguments)?;
            let resource_type = parse_resource_type(&args.resource_type, &request.id)?;

            let parsed_parameters = ParsedParameters::try_from(
                &args.search_parameters.unwrap_or_default(),
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

            let json_value = serde_json::to_value(&result).map_err(|e| {
                OperationOutcomeError::error(
                    IssueType::processing(),
                    format!("Failed to serialize search result: '{}'", e),
                )
            })?;

            success_result(&json_value)
        }
        GET_SEARCH_PARAMETERS_TOOL_NAME => {
            let capabilities = ctx.client.capabilities(ctx.clone()).await?;
            let resource_capability_statement = capabilities
                .rest
                .unwrap_or_default()
                .into_iter()
                .filter_map(|rest| rest.resource)
                .flatten()
                .find(|r| {
                    let rc_type = r.type_.as_str();
                    rc_type.unwrap_or_default()
                        == request
                            .params
                            .arguments
                            .as_ref()
                            .and_then(|args| args.get("resourceType"))
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                });

            let Some(resource_capability_statement_params) = resource_capability_statement
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

            let parameters = search_tool_parameters(resource_capability_statement_params);
            success_result(&parameters)
        }
        R4_READ_TOOL_NAME => {
            let args: FHIRReadArguments = parse_arguments(request.params.arguments)?;
            let resource_type = parse_resource_type(&args.resource_type, &request.id)?;

            let result = ctx
                .client
                .read(ctx.clone(), resource_type, args.id.clone())
                .await?;

            match result {
                Some(resource) => {
                    let json_value = resource_to_json(&resource)?;
                    success_result(&json_value)
                }
                None => Err(MCPError {
                    id: request.id.clone(),
                    jsonrpc: "2.0".to_string(),
                    error: MCPErrorDetail {
                        code: 404,
                        message: format!("Resource {}/{} not found", args.resource_type, args.id),
                        data: None,
                    },
                }),
            }
        }
        R4_VREAD_TOOL_NAME => {
            let args: FHIRVReadArguments = parse_arguments(request.params.arguments)?;
            let resource_type = parse_resource_type(&args.resource_type, &request.id)?;

            let result = ctx
                .client
                .vread(
                    ctx.clone(),
                    resource_type,
                    args.id.clone(),
                    args.version_id.clone(),
                )
                .await?;

            match result {
                Some(resource) => {
                    let json_value = resource_to_json(&resource)?;
                    success_result(&json_value)
                }
                None => Err(MCPError {
                    id: request.id.clone(),
                    jsonrpc: "2.0".to_string(),
                    error: MCPErrorDetail {
                        code: 404,
                        message: format!(
                            "Resource {}/{}/_history/{} not found",
                            args.resource_type, args.id, args.version_id
                        ),
                        data: None,
                    },
                }),
            }
        }
        R4_CREATE_TOOL_NAME => {
            let args: FHIRCreateArguments = parse_arguments(request.params.arguments)?;
            let resource_type = parse_resource_type(&args.resource_type, &request.id)?;
            let resource = json_to_resource(args.resource)?;

            let result = ctx
                .client
                .create(ctx.clone(), resource_type, resource)
                .await?;

            let json_value = resource_to_json(&result)?;
            success_result(&json_value)
        }
        R4_UPDATE_TOOL_NAME => {
            let args: FHIRUpdateArguments = parse_arguments(request.params.arguments)?;
            let resource_type = parse_resource_type(&args.resource_type, &request.id)?;
            let resource = json_to_resource(args.resource)?;

            let result = ctx
                .client
                .update(ctx.clone(), resource_type, args.id, resource)
                .await?;

            let json_value = resource_to_json(&result)?;
            success_result(&json_value)
        }
        R4_PATCH_TOOL_NAME => {
            let args: FHIRPatchArguments = parse_arguments(request.params.arguments)?;
            let resource_type = parse_resource_type(&args.resource_type, &request.id)?;

            let patches: Patch = serde_json::from_value(args.patches).map_err(|e| MCPError {
                id: request.id.clone(),
                jsonrpc: "2.0".to_string(),
                error: MCPErrorDetail {
                    code: 400,
                    message: format!("Invalid JSON Patch document: '{}'", e),
                    data: None,
                },
            })?;

            let result = ctx
                .client
                .patch(ctx.clone(), resource_type, args.id, patches)
                .await?;

            let json_value = resource_to_json(&result)?;
            success_result(&json_value)
        }
        R4_DELETE_TOOL_NAME => {
            let args: FHIRDeleteArguments = parse_arguments(request.params.arguments)?;
            let resource_type = parse_resource_type(&args.resource_type, &request.id)?;

            ctx.client
                .delete_instance(ctx.clone(), resource_type, args.id.clone())
                .await?;

            success_result(&serde_json::json!({
                "success": true,
                "message": format!("Resource {}/{} deleted successfully", args.resource_type, args.id),
            }))
        }
        R4_HISTORY_INSTANCE_TOOL_NAME => {
            let args: FHIRHistoryInstanceArguments = parse_arguments(request.params.arguments)?;
            let resource_type = parse_resource_type(&args.resource_type, &request.id)?;
            let params = history_params(args.count, args.since)?;

            let result = ctx
                .client
                .history_instance(ctx.clone(), resource_type, args.id, params)
                .await?;

            let json_value = serde_json::to_value(&result).map_err(|e| {
                OperationOutcomeError::error(
                    IssueType::processing(),
                    format!("Failed to serialize history result: '{}'", e),
                )
            })?;

            success_result(&json_value)
        }
        R4_HISTORY_TYPE_TOOL_NAME => {
            let args: FHIRHistoryTypeArguments = parse_arguments(request.params.arguments)?;
            let resource_type = parse_resource_type(&args.resource_type, &request.id)?;
            let params = history_params(args.count, args.since)?;

            let result = ctx
                .client
                .history_type(ctx.clone(), resource_type, params)
                .await?;

            let json_value = serde_json::to_value(&result).map_err(|e| {
                OperationOutcomeError::error(
                    IssueType::processing(),
                    format!("Failed to serialize history result: '{}'", e),
                )
            })?;

            success_result(&json_value)
        }
        R4_CAPABILITIES_TOOL_NAME => {
            let capabilities = ctx.client.capabilities(ctx.clone()).await?;

            let json_value = serde_json::to_value(&capabilities).map_err(|e| {
                OperationOutcomeError::error(
                    IssueType::processing(),
                    format!("Failed to serialize capabilities: '{}'", e),
                )
            })?;

            success_result(&json_value)
        }
        R4_TRANSACTION_TOOL_NAME => {
            let args: FHIRBundleArguments = parse_arguments(request.params.arguments)?;

            let bundle: Bundle = serde_json::from_value(args.bundle).map_err(|e| MCPError {
                id: request.id.clone(),
                jsonrpc: "2.0".to_string(),
                error: MCPErrorDetail {
                    code: 400,
                    message: format!("Invalid transaction Bundle: '{}'", e),
                    data: None,
                },
            })?;

            let result = ctx.client.transaction(ctx.clone(), bundle).await?;

            let json_value = serde_json::to_value(&result).map_err(|e| {
                OperationOutcomeError::error(
                    IssueType::processing(),
                    format!("Failed to serialize transaction result: '{}'", e),
                )
            })?;

            success_result(&json_value)
        }
        R4_BATCH_TOOL_NAME => {
            let args: FHIRBundleArguments = parse_arguments(request.params.arguments)?;

            let bundle: Bundle = serde_json::from_value(args.bundle).map_err(|e| MCPError {
                id: request.id.clone(),
                jsonrpc: "2.0".to_string(),
                error: MCPErrorDetail {
                    code: 400,
                    message: format!("Invalid batch Bundle: '{}'", e),
                    data: None,
                },
            })?;

            let result = ctx.client.batch(ctx.clone(), bundle).await?;

            let json_value = serde_json::to_value(&result).map_err(|e| {
                OperationOutcomeError::error(
                    IssueType::processing(),
                    format!("Failed to serialize batch result: '{}'", e),
                )
            })?;

            success_result(&json_value)
        }
        _ => Err(MCPError {
            id: request.id.clone(),
            jsonrpc: "2.0".to_string(),
            error: MCPErrorDetail {
                code: 400,
                message: format!("Unknown tool name: '{}'", request.params.name),
                data: None,
            },
        }),
    }
}
