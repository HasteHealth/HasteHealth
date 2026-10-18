use crate::{
    fhir_client::ServerCTX,
    mcp::{
        error::{MCPError, MCPErrorDetail},
        operations::{
            GET_RESOURCE_SCHEMA_TOOL_NAME, GET_SEARCH_PARAMETERS_TOOL_NAME, OPERATION_TOOL_PREFIX,
            R4_BATCH_TOOL_NAME, R4_CAPABILITIES_TOOL_NAME, R4_CREATE_TOOL_NAME,
            R4_DELETE_TOOL_NAME, R4_HISTORY_INSTANCE_TOOL_NAME, R4_HISTORY_TYPE_TOOL_NAME,
            R4_PATCH_TOOL_NAME, R4_READ_TOOL_NAME, R4_SEARCH_TOOL_NAME, R4_TRANSACTION_TOOL_NAME,
            R4_UPDATE_TOOL_NAME, R4_VREAD_TOOL_NAME, discover_operations, invoke_operation,
            schema_base_url, search_tool_parameters,
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
struct FHIRGetResourceSchemaArguments {
    #[serde(rename = "resourceType")]
    resource_type: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct FHIRGetSearchParametersArguments {
    #[serde(rename = "resourceType")]
    resource_type: String,
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

/// Resource types this project serves, from its CapabilityStatement. Empty
/// when it cannot be read, which only costs the suggestion.
async fn supported_resource_types<
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>(
    ctx: &Arc<ServerCTX<Client>>,
) -> Vec<String> {
    let Ok(capabilities) = ctx.client.capabilities(ctx.clone()).await else {
        return Vec::new();
    };

    capabilities
        .rest
        .unwrap_or_default()
        .into_iter()
        .filter_map(|rest| rest.resource)
        .flatten()
        .filter_map(|resource| resource.type_.as_str().map(str::to_string))
        .collect()
}

/// Levenshtein distance between two names.
fn edit_distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut previous: Vec<usize> = (0..=b.len()).collect();

    for (i, ca) in a.chars().enumerate() {
        let mut current = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            let substitution = previous[j] + usize::from(ca != *cb);
            current.push(substitution.min(previous[j + 1] + 1).min(current[j] + 1));
        }
        previous = current;
    }

    previous[b.len()]
}

/// The supported type a mistyped name most likely meant: the same name in
/// another case, else the nearest within a few edits.
fn closest_resource_type<'a>(requested: &str, candidates: &'a [String]) -> Option<&'a str> {
    if let Some(same) = candidates
        .iter()
        .find(|candidate| candidate.eq_ignore_ascii_case(requested))
    {
        return Some(same.as_str());
    }

    let lower = requested.to_ascii_lowercase();
    let threshold = (requested.len() / 3).clamp(1, 3);
    candidates
        .iter()
        .map(|candidate| {
            (
                edit_distance(&lower, &candidate.to_ascii_lowercase()),
                candidate,
            )
        })
        .filter(|(distance, _)| *distance <= threshold)
        .min_by_key(|(distance, _)| *distance)
        .map(|(_, candidate)| candidate.as_str())
}

/// Helper: parse resource type string into `ResourceType`. A miss names the
/// closest supported type, since the argument schema no longer lists them.
async fn parse_resource_type<
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>(
    ctx: &Arc<ServerCTX<Client>>,
    resource_type: &str,
    request_id: &Option<crate::mcp::schemas::types::RequestId>,
) -> Result<ResourceType, MCPError<serde_json::Value>> {
    if let Ok(parsed) = ResourceType::try_from(resource_type) {
        return Ok(parsed);
    }

    let supported = supported_resource_types(ctx).await;
    let message = match closest_resource_type(resource_type, &supported) {
        Some(suggestion) => format!(
            "Invalid resource type '{resource_type}'. Did you mean '{suggestion}'? \
             Resource types are case-sensitive."
        ),
        None => format!(
            "Invalid resource type '{resource_type}'. Use a FHIR R4 type such as Patient; \
             the '{R4_CAPABILITIES_TOOL_NAME}' tool lists every type this project supports."
        ),
    };

    Err(MCPError {
        id: request_id.clone(),
        jsonrpc: "2.0".to_string(),
        error: MCPErrorDetail {
            code: 400,
            message,
            data: None,
        },
    })
}

/// Helper: parse tool arguments from JSON value.
///
/// A failure names the tool, the field serde rejected and the shape the tool
/// expects, so a model can correct the call rather than guess at it.
fn parse_arguments<T: serde::de::DeserializeOwned>(
    tool_name: &str,
    arguments: Option<serde_json::Value>,
    example: serde_json::Value,
) -> Result<T, MCPError<serde_json::Value>> {
    // Absent arguments read as an empty object, so the error names the first
    // missing field instead of saying a struct was expected.
    let arguments = match arguments {
        Some(serde_json::Value::Null) | None => serde_json::json!({}),
        Some(arguments) => arguments,
    };

    serde_path_to_error::deserialize::<_, T>(arguments).map_err(|e| {
        OperationOutcomeError::error(
            IssueType::invalid(),
            format!(
                "Invalid arguments for '{tool_name}': {}. Expected an object like {example}",
                describe_json_error(e)
            ),
        )
        .into()
    })
}

/// A deserialization error with the path of the element it happened at, so
/// "invalid FHIR date" says which date (`birthDate`, `name[0].period.start`).
fn describe_json_error(error: serde_path_to_error::Error<serde_json::Error>) -> String {
    let path = error.path().to_string();
    let inner = error.into_inner();

    // "." is the root: the message already names the field or the type.
    if path == "." {
        inner.to_string()
    } else {
        format!("{inner} at {path}")
    }
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
    serde_path_to_error::deserialize::<_, Resource>(value).map_err(|e| {
        OperationOutcomeError::error(
            IssueType::invalid(),
            format!(
                "Invalid 'resource': {}. The '{GET_RESOURCE_SCHEMA_TOOL_NAME}' tool gives the \
                 exact schema of a resource type",
                describe_json_error(e)
            ),
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
    ParsedParameters::try_from(&params).map_err(|e| {
        OperationOutcomeError::error(
            IssueType::invalid(),
            format!(
                "Invalid history parameters: {e}. _count is a positive integer and _since an \
                 instant such as 2024-01-01T00:00:00Z"
            ),
        )
        .into()
    })
}

pub async fn tools_call<
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>(
    ctx: Arc<ServerCTX<Client>>,
    request: CallToolRequest,
    api_uri: &str,
) -> Result<CallToolResult, MCPError<serde_json::Value>> {
    match request.params.name.as_str() {
        R4_SEARCH_TOOL_NAME => {
            let args: FHIRSearchArguments = parse_arguments(
                R4_SEARCH_TOOL_NAME,
                request.params.arguments,
                serde_json::json!({"resourceType": "Observation", "search_parameters": {"code": "http://loinc.org|8867-4", "_count": "20"}}),
            )?;
            let resource_type = parse_resource_type(&ctx, &args.resource_type, &request.id).await?;

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
            let args: FHIRGetSearchParametersArguments = parse_arguments(
                GET_SEARCH_PARAMETERS_TOOL_NAME,
                request.params.arguments,
                serde_json::json!({"resourceType": "Patient"}),
            )?;
            parse_resource_type(&ctx, &args.resource_type, &request.id).await?;

            let capabilities = ctx.client.capabilities(ctx.clone()).await?;
            let resource_capability_statement = capabilities
                .rest
                .unwrap_or_default()
                .into_iter()
                .filter_map(|rest| rest.resource)
                .flatten()
                .find(|r| r.type_.as_str() == Some(args.resource_type.as_str()));

            let Some(resource_capability_statement_params) = resource_capability_statement
                .as_ref()
                .and_then(|rc| rc.searchParam.as_ref())
            else {
                return Err(MCPError {
                    id: request.id.clone(),
                    jsonrpc: "2.0".to_string(),
                    error: MCPErrorDetail {
                        code: 400,
                        message: format!(
                            "Resource type '{}' is not served by this project; the '{}' tool \
                             lists the supported types",
                            args.resource_type, R4_CAPABILITIES_TOOL_NAME
                        ),
                        data: None,
                    },
                });
            };

            let parameters = search_tool_parameters(resource_capability_statement_params);
            success_result(&parameters)
        }
        GET_RESOURCE_SCHEMA_TOOL_NAME => {
            let args: FHIRGetResourceSchemaArguments = parse_arguments(
                GET_RESOURCE_SCHEMA_TOOL_NAME,
                request.params.arguments,
                serde_json::json!({"resourceType": "Patient"}),
            )?;
            // Validate it's a real resource type before fetching.
            parse_resource_type(&ctx, &args.resource_type, &request.id).await?;

            let url = format!("{}/{}", schema_base_url(api_uri), args.resource_type);

            let response = reqwest::get(&url).await.map_err(|e| MCPError {
                id: request.id.clone(),
                jsonrpc: "2.0".to_string(),
                error: MCPErrorDetail {
                    code: 502,
                    message: format!("Failed to fetch schema for '{}': {}", args.resource_type, e),
                    data: None,
                },
            })?;

            if !response.status().is_success() {
                return Err(MCPError {
                    id: request.id.clone(),
                    jsonrpc: "2.0".to_string(),
                    error: MCPErrorDetail {
                        code: 502,
                        message: format!(
                            "Schema endpoint returned status {} for resourceType '{}'",
                            response.status(),
                            args.resource_type
                        ),
                        data: None,
                    },
                });
            }

            let schema: serde_json::Value = response.json().await.map_err(|e| MCPError {
                id: request.id.clone(),
                jsonrpc: "2.0".to_string(),
                error: MCPErrorDetail {
                    code: 502,
                    message: format!(
                        "Invalid schema response for '{}': {}",
                        args.resource_type, e
                    ),
                    data: None,
                },
            })?;

            success_result(&schema)
        }
        R4_READ_TOOL_NAME => {
            let args: FHIRReadArguments = parse_arguments(
                R4_READ_TOOL_NAME,
                request.params.arguments,
                serde_json::json!({"resourceType": "Patient", "id": "123"}),
            )?;
            let resource_type = parse_resource_type(&ctx, &args.resource_type, &request.id).await?;

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
            let args: FHIRVReadArguments = parse_arguments(
                R4_VREAD_TOOL_NAME,
                request.params.arguments,
                serde_json::json!({"resourceType": "Patient", "id": "123", "versionId": "2"}),
            )?;
            let resource_type = parse_resource_type(&ctx, &args.resource_type, &request.id).await?;

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
            let args: FHIRCreateArguments = parse_arguments(
                R4_CREATE_TOOL_NAME,
                request.params.arguments,
                serde_json::json!({"resourceType": "Patient", "resource": {"resourceType": "Patient", "name": [{"family": "Smith"}]}}),
            )?;
            let resource_type = parse_resource_type(&ctx, &args.resource_type, &request.id).await?;
            let resource = json_to_resource(args.resource)?;

            let result = ctx
                .client
                .create(ctx.clone(), resource_type, resource)
                .await?;

            let json_value = resource_to_json(&result)?;
            success_result(&json_value)
        }
        R4_UPDATE_TOOL_NAME => {
            let args: FHIRUpdateArguments = parse_arguments(
                R4_UPDATE_TOOL_NAME,
                request.params.arguments,
                serde_json::json!({"resourceType": "Patient", "id": "123", "resource": {"resourceType": "Patient", "id": "123", "active": true}}),
            )?;
            let resource_type = parse_resource_type(&ctx, &args.resource_type, &request.id).await?;
            let resource = json_to_resource(args.resource)?;

            let result = ctx
                .client
                .update(ctx.clone(), resource_type, args.id, resource)
                .await?;

            let json_value = resource_to_json(&result)?;
            success_result(&json_value)
        }
        R4_PATCH_TOOL_NAME => {
            let args: FHIRPatchArguments = parse_arguments(
                R4_PATCH_TOOL_NAME,
                request.params.arguments,
                serde_json::json!({"resourceType": "Patient", "id": "123", "patches": [{"op": "replace", "path": "/active", "value": false}]}),
            )?;
            let resource_type = parse_resource_type(&ctx, &args.resource_type, &request.id).await?;

            let patches: Patch = serde_json::from_value(args.patches).map_err(|e| MCPError {
                id: request.id.clone(),
                jsonrpc: "2.0".to_string(),
                error: MCPErrorDetail {
                    code: 400,
                    message: format!(
                        "Invalid 'patches': {e}. Expected an array of RFC 6902 operations such as \
                         [{{\"op\": \"replace\", \"path\": \"/active\", \"value\": false}}]"
                    ),
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
            let args: FHIRDeleteArguments = parse_arguments(
                R4_DELETE_TOOL_NAME,
                request.params.arguments,
                serde_json::json!({"resourceType": "Patient", "id": "123"}),
            )?;
            let resource_type = parse_resource_type(&ctx, &args.resource_type, &request.id).await?;

            ctx.client
                .delete_instance(ctx.clone(), resource_type, args.id.clone())
                .await?;

            success_result(&serde_json::json!({
                "success": true,
                "message": format!("Resource {}/{} deleted successfully", args.resource_type, args.id),
            }))
        }
        R4_HISTORY_INSTANCE_TOOL_NAME => {
            let args: FHIRHistoryInstanceArguments = parse_arguments(
                R4_HISTORY_INSTANCE_TOOL_NAME,
                request.params.arguments,
                serde_json::json!({"resourceType": "Patient", "id": "123", "_count": "10"}),
            )?;
            let resource_type = parse_resource_type(&ctx, &args.resource_type, &request.id).await?;
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
            let args: FHIRHistoryTypeArguments = parse_arguments(
                R4_HISTORY_TYPE_TOOL_NAME,
                request.params.arguments,
                serde_json::json!({"resourceType": "Patient", "_since": "2024-01-01T00:00:00Z"}),
            )?;
            let resource_type = parse_resource_type(&ctx, &args.resource_type, &request.id).await?;
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
            let args: FHIRBundleArguments = parse_arguments(
                R4_TRANSACTION_TOOL_NAME,
                request.params.arguments,
                serde_json::json!({"bundle": {"resourceType": "Bundle", "type": "transaction", "entry": [{"request": {"method": "POST", "url": "Patient"}, "resource": {"resourceType": "Patient"}}]}}),
            )?;

            let bundle: Bundle = serde_json::from_value(args.bundle).map_err(|e| MCPError {
                id: request.id.clone(),
                jsonrpc: "2.0".to_string(),
                error: MCPErrorDetail {
                    code: 400,
                    message: format!(
                        "Invalid 'bundle': {e}. Expected a FHIR Bundle of type 'transaction' whose \
                         entries each carry request.method and request.url, plus a resource for \
                         POST and PUT"
                    ),
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
            let args: FHIRBundleArguments = parse_arguments(
                R4_BATCH_TOOL_NAME,
                request.params.arguments,
                serde_json::json!({"bundle": {"resourceType": "Bundle", "type": "batch", "entry": [{"request": {"method": "GET", "url": "Patient/123"}}]}}),
            )?;

            let bundle: Bundle = serde_json::from_value(args.bundle).map_err(|e| MCPError {
                id: request.id.clone(),
                jsonrpc: "2.0".to_string(),
                error: MCPErrorDetail {
                    code: 400,
                    message: format!(
                        "Invalid 'bundle': {e}. Expected a FHIR Bundle of type 'batch' whose \
                         entries each carry request.method and request.url, plus a resource for \
                         POST and PUT"
                    ),
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
        // Anything else is either a FHIR operation exposed as a tool -- built-in
        // or user-created -- or genuinely unknown.
        name if name.starts_with(OPERATION_TOOL_PREFIX) => {
            let operations = discover_operations(ctx.clone()).await?;

            let Some(operation) = operations.iter().find(|o| o.tool_name == name) else {
                return Err(MCPError {
                    id: request.id.clone(),
                    jsonrpc: "2.0".to_string(),
                    error: MCPErrorDetail {
                        code: 404,
                        message: format!("Unknown operation tool: '{name}'"),
                        data: None,
                    },
                });
            };

            let result = invoke_operation(
                ctx.clone(),
                operation,
                request.params.arguments.clone(),
                &request.id,
            )
            .await?;

            let json_value = resource_to_json(&result)?;
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
