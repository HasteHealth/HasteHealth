//! Invocation of FHIR operations exposed as MCP tools.
//!
//! Converts the flat JSON arguments an MCP client sends into the FHIR
//! `Parameters` resource operations expect, then dispatches to the right
//! invocation level.

use crate::{
    fhir_client::ServerCTX,
    mcp::{
        error::{MCPError, MCPErrorDetail},
        operations::operation_tools::{OperationLevel, OperationTool},
        schemas::types::RequestId,
    },
};
use haste_fhir_client::FHIRClient;
use haste_fhir_model::r4::generated::{
    resources::{OperationDefinitionParameter, Parameters, Resource, ResourceType},
    terminology::IssueType,
};
use haste_fhir_operation_error::OperationOutcomeError;
use serde_json::{Map, Value};
use std::sync::Arc;

/// Arguments that target the invocation rather than feed the operation itself.
const TARGETING_ARGUMENTS: [&str; 2] = ["id", "resourceType"];

/// Map a FHIR parameter type to the `value[x]` key used in Parameters JSON.
///
/// Returning `None` means the value is a resource rather than a datatype, and
/// belongs under `resource` instead of `value[x]`.
fn value_key(fhir_type: &str) -> Option<String> {
    // Resources are carried in `resource`, not `value[x]`.
    if fhir_type == "Resource" || ResourceType::try_from(fhir_type).is_ok() {
        return None;
    }

    let mut chars = fhir_type.chars();
    let capitalized = match chars.next() {
        Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
        None => return None,
    };

    Some(format!("value{capitalized}"))
}

/// Build one `Parameters.parameter` entry from a JSON argument.
fn parameter_entry(name: &str, fhir_type: Option<&str>, value: Value) -> Value {
    let mut entry = Map::new();
    entry.insert("name".to_string(), Value::String(name.to_string()));

    match fhir_type.and_then(value_key) {
        Some(key) => {
            entry.insert(key, value);
        }
        None => {
            // Either a resource-typed parameter or an untyped one. A JSON object
            // carrying a resourceType is a resource; anything else is best
            // represented as a string so the operation still receives it.
            if value.get("resourceType").is_some() {
                entry.insert("resource".to_string(), value);
            } else if let Value::String(s) = &value {
                entry.insert("valueString".to_string(), Value::String(s.clone()));
            } else {
                entry.insert("valueString".to_string(), Value::String(value.to_string()));
            }
        }
    }

    Value::Object(entry)
}

/// Look up an operation's declared input parameter by name.
fn find_input_parameter<'a>(
    operation: &'a OperationTool,
    name: &str,
) -> Option<&'a OperationDefinitionParameter> {
    operation
        .definition
        .parameter
        .as_deref()
        .unwrap_or_default()
        .iter()
        .find(|p| p.use_.as_str() == Some("in") && p.name.value.as_deref() == Some(name))
}

/// Convert MCP tool arguments into the FHIR `Parameters` resource to invoke with.
pub fn arguments_to_parameters(
    operation: &OperationTool,
    arguments: &Map<String, Value>,
) -> Result<Parameters, MCPError<Value>> {
    let mut entries: Vec<Value> = Vec::new();

    for (name, value) in arguments {
        if TARGETING_ARGUMENTS.contains(&name.as_str()) {
            continue;
        }

        let declared = find_input_parameter(operation, name);
        let fhir_type = declared
            .and_then(|p| p.type_.as_ref())
            .and_then(|t| t.as_str());

        // A repeating parameter arrives as an array and becomes one entry per item.
        match value {
            Value::Array(items) => {
                for item in items {
                    entries.push(parameter_entry(name, fhir_type, item.clone()));
                }
            }
            _ => entries.push(parameter_entry(name, fhir_type, value.clone())),
        }
    }

    let parameters_json = serde_json::json!({
        "resourceType": "Parameters",
        "parameter": entries,
    });

    serde_json::from_value(parameters_json).map_err(|e| {
        OperationOutcomeError::error(
            IssueType::invalid(),
            format!("Failed to build Parameters for operation invocation: '{e}'"),
        )
        .into()
    })
}

/// Decide which level to invoke at, given the arguments and what the operation supports.
fn resolve_level(
    operation: &OperationTool,
    arguments: &Map<String, Value>,
) -> Option<OperationLevel> {
    let levels = operation.levels();
    let has_id = arguments.get("id").and_then(Value::as_str).is_some();

    // An explicit id means instance level, when the operation allows it.
    if has_id && levels.contains(&OperationLevel::Instance) {
        return Some(OperationLevel::Instance);
    }

    // Otherwise prefer type level when a resource type is determinable,
    // falling back to system level.
    if levels.contains(&OperationLevel::Type) && !operation.resource_types().is_empty() {
        return Some(OperationLevel::Type);
    }

    if levels.contains(&OperationLevel::System) {
        return Some(OperationLevel::System);
    }

    levels.first().copied()
}

/// Determine the resource type to invoke a type/instance-level operation against.
fn resolve_resource_type(
    operation: &OperationTool,
    arguments: &Map<String, Value>,
    request_id: &Option<RequestId>,
) -> Result<ResourceType, MCPError<Value>> {
    let declared = operation.resource_types();

    if let Some(requested) = arguments.get("resourceType").and_then(Value::as_str) {
        let parsed = ResourceType::try_from(requested).map_err(|_| MCPError {
            id: request_id.clone(),
            jsonrpc: "2.0".to_string(),
            error: MCPErrorDetail {
                code: 400,
                message: format!("Invalid resource type: '{requested}'"),
                data: None,
            },
        })?;

        // Keep the caller inside the operation's declared surface.
        if !declared.is_empty() && !declared.contains(&parsed) {
            let names: Vec<&str> = declared.iter().map(|r| r.as_ref()).collect();
            return Err(MCPError {
                id: request_id.clone(),
                jsonrpc: "2.0".to_string(),
                error: MCPErrorDetail {
                    code: 400,
                    message: format!(
                        "Operation is not defined for resource type '{requested}'. Supported: {}",
                        names.join(", ")
                    ),
                    data: None,
                },
            });
        }

        return Ok(parsed);
    }

    // No explicit type: unambiguous only when the operation names exactly one.
    match declared.len() {
        1 => Ok(declared[0].clone()),
        0 => Err(MCPError {
            id: request_id.clone(),
            jsonrpc: "2.0".to_string(),
            error: MCPErrorDetail {
                code: 400,
                message: "Operation is not defined against a resource type".to_string(),
                data: None,
            },
        }),
        _ => {
            let names: Vec<&str> = declared.iter().map(|r| r.as_ref()).collect();
            Err(MCPError {
                id: request_id.clone(),
                jsonrpc: "2.0".to_string(),
                error: MCPErrorDetail {
                    code: 400,
                    message: format!(
                        "'resourceType' is required for this operation. Supported: {}",
                        names.join(", ")
                    ),
                    data: None,
                },
            })
        }
    }
}

/// Invoke an operation on behalf of an MCP `tools/call`.
pub async fn invoke_operation<
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>(
    ctx: Arc<ServerCTX<Client>>,
    operation: &OperationTool,
    arguments: Option<Value>,
    request_id: &Option<RequestId>,
) -> Result<Resource, MCPError<Value>> {
    let arguments = match arguments {
        Some(Value::Object(map)) => map,
        Some(Value::Null) | None => Map::new(),
        Some(_) => {
            return Err(OperationOutcomeError::error(
                IssueType::invalid(),
                "Tool arguments must be a JSON object".to_string(),
            )
            .into());
        }
    };

    let code = operation.definition.code.value.clone().unwrap_or_default();

    let parameters = arguments_to_parameters(operation, &arguments)?;

    let Some(level) = resolve_level(operation, &arguments) else {
        return Err(OperationOutcomeError::error(
            IssueType::not_supported(),
            format!("Operation '${code}' declares no invocable level"),
        )
        .into());
    };

    let result = match level {
        OperationLevel::Instance => {
            let resource_type = resolve_resource_type(operation, &arguments, request_id)?;
            let Some(id) = arguments.get("id").and_then(Value::as_str) else {
                return Err(MCPError {
                    id: request_id.clone(),
                    jsonrpc: "2.0".to_string(),
                    error: MCPErrorDetail {
                        code: 400,
                        message: format!("'id' is required to invoke '${code}' at instance level"),
                        data: None,
                    },
                });
            };

            ctx.client
                .invoke_instance(
                    ctx.clone(),
                    resource_type,
                    id.to_string(),
                    code.clone(),
                    parameters,
                )
                .await?
        }
        OperationLevel::Type => {
            let resource_type = resolve_resource_type(operation, &arguments, request_id)?;

            ctx.client
                .invoke_type(ctx.clone(), resource_type, code.clone(), parameters)
                .await?
        }
        OperationLevel::System => {
            ctx.client
                .invoke_system(ctx.clone(), code.clone(), parameters)
                .await?
        }
    };

    Ok(result)
}
