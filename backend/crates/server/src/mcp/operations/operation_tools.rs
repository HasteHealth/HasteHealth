//! Exposes FHIR operations as individual MCP tools.
//!
//! Every invocable operation on this server is an `OperationDefinition` in the
//! repository -- the built-in ones are loaded from the embedded artifacts, and the
//! ones a project author writes are ordinary resources executed through Deno. Both
//! are discovered the same way here, so a user-created operation shows up in
//! `tools/list` with a real typed schema without any extra registration.

use crate::{
    fhir_client::ServerCTX,
    mcp::{error::MCPError, schemas::types::Tool},
};
use haste_fhir_client::FHIRClient;
use haste_fhir_model::r4::generated::resources::{
    OperationDefinition, OperationDefinitionParameter, Resource, ResourceType,
};
use haste_fhir_operation_error::OperationOutcomeError;
use serde_json::json;
use std::sync::Arc;

/// Prefix for every generated per-operation tool name.
pub const OPERATION_TOOL_PREFIX: &str = "fhir_op_";

/// The level an operation is invoked at, mirroring the `system`/`type`/`instance`
/// flags on `OperationDefinition`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationLevel {
    System,
    Type,
    Instance,
}

/// An operation paired with the MCP tool name it is advertised under.
pub struct OperationTool {
    pub definition: OperationDefinition,
    pub tool_name: String,
}

impl OperationTool {
    /// Levels this operation may be invoked at, most specific first.
    pub fn levels(&self) -> Vec<OperationLevel> {
        let mut levels = Vec::new();
        if self.definition.instance.value.unwrap_or(false) {
            levels.push(OperationLevel::Instance);
        }
        if self.definition.type_.value.unwrap_or(false) {
            levels.push(OperationLevel::Type);
        }
        if self.definition.system.value.unwrap_or(false) {
            levels.push(OperationLevel::System);
        }
        levels
    }

    /// Resource types this operation is defined against.
    pub fn resource_types(&self) -> Vec<ResourceType> {
        self.definition
            .resource
            .as_deref()
            .unwrap_or_default()
            .iter()
            .filter_map(|r| r.as_str())
            .filter_map(|r| ResourceType::try_from(r).ok())
            .collect()
    }

    /// Whether invoking this operation can change server state. Operations that
    /// omit `affectsState` are treated as read-only, per the FHIR default.
    pub fn affects_state(&self) -> bool {
        self.definition
            .affectsState
            .as_ref()
            .and_then(|a| a.value)
            .unwrap_or(false)
    }
}

/// Turn an operation `code` into a stable MCP tool name.
///
/// Tool names are restricted to `[a-z0-9_]` so that codes containing `-` or `.`
/// (common in user-authored operations) stay valid.
pub fn operation_tool_name(code: &str) -> String {
    let sanitized: String = code
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();

    format!("{OPERATION_TOOL_PREFIX}{sanitized}")
}

/// Map a FHIR parameter type onto a JSON Schema fragment.
///
/// Primitives map to their JSON counterparts; everything else is a complex FHIR
/// type or a resource, which the model supplies as an object.
fn parameter_type_schema(
    fhir_type: Option<&str>,
    documentation: Option<&str>,
) -> serde_json::Value {
    let mut schema = match fhir_type {
        Some("boolean") => json!({ "type": "boolean" }),
        Some("integer" | "positiveInt" | "unsignedInt") => json!({ "type": "integer" }),
        Some("decimal") => json!({ "type": "number" }),
        Some(
            "string" | "code" | "uri" | "url" | "canonical" | "oid" | "id" | "uuid" | "markdown"
            | "base64Binary" | "date" | "dateTime" | "instant" | "time",
        ) => json!({ "type": "string" }),
        // Complex datatypes and resources are passed through as JSON objects.
        _ => json!({ "type": "object" }),
    };

    if let Some(documentation) = documentation
        && let Some(object) = schema.as_object_mut()
    {
        object.insert("description".to_string(), json!(documentation));
    }

    if let Some(fhir_type) = fhir_type
        && let Some(object) = schema.as_object_mut()
    {
        object.insert("x-fhir-type".to_string(), json!(fhir_type));
    }

    schema
}

/// Build the JSON Schema for one input parameter, accounting for cardinality.
fn input_parameter_schema(parameter: &OperationDefinitionParameter) -> serde_json::Value {
    let fhir_type = parameter.type_.as_ref().and_then(|t| t.as_str());
    let documentation = parameter
        .documentation
        .as_ref()
        .and_then(|d| d.value.as_deref());

    let base = parameter_type_schema(fhir_type, documentation);

    // `max` of "*" or any value above one means the parameter repeats.
    let repeats = match parameter.max.value.as_deref() {
        Some("*") => true,
        Some(max) => max.parse::<u32>().map(|m| m > 1).unwrap_or(false),
        None => false,
    };

    if repeats {
        json!({
            "type": "array",
            "items": base,
            "description": documentation,
        })
    } else {
        base
    }
}

/// Build the MCP input schema for an operation.
///
/// Instance-level operations need an `id`, and operations defined against more
/// than one resource type need to be told which one to target, so both are
/// surfaced as explicit arguments alongside the operation's own parameters.
fn operation_input_schema(operation: &OperationTool) -> serde_json::Value {
    let mut properties = serde_json::Map::new();
    let mut required: Vec<String> = Vec::new();

    let levels = operation.levels();
    let resource_types = operation.resource_types();

    if levels.contains(&OperationLevel::Instance) {
        let id_schema = if levels.len() == 1 {
            // Instance is the only level, so an id is mandatory.
            required.push("id".to_string());
            json!({
                "type": "string",
                "description": "Logical ID of the resource to invoke the operation on.",
            })
        } else {
            json!({
                "type": "string",
                "description": "Logical ID of the resource to invoke the operation on. \
                                Omit to invoke the operation at the type or system level.",
            })
        };
        properties.insert("id".to_string(), id_schema);
    }

    // Only ask for a resource type when the choice is genuinely ambiguous.
    if !resource_types.is_empty()
        && (levels.contains(&OperationLevel::Instance) || levels.contains(&OperationLevel::Type))
    {
        let type_names: Vec<&str> = resource_types.iter().map(|r| r.as_ref()).collect();

        if type_names.len() > 1 {
            properties.insert(
                "resourceType".to_string(),
                json!({
                    "type": "string",
                    "enum": type_names,
                    "description": "Resource type to invoke the operation against.",
                }),
            );
        }
    }

    for parameter in operation
        .definition
        .parameter
        .as_deref()
        .unwrap_or_default()
        .iter()
        .filter(|p| p.use_.as_str() == Some("in"))
    {
        let Some(name) = parameter.name.value.as_deref() else {
            continue;
        };

        // Don't let an operation parameter shadow the invocation targeting args.
        if properties.contains_key(name) {
            continue;
        }

        properties.insert(name.to_string(), input_parameter_schema(parameter));

        if parameter.min.value.unwrap_or(0) > 0 {
            required.push(name.to_string());
        }
    }

    let mut schema = json!({
        "type": "object",
        "properties": serde_json::Value::Object(properties),
    });

    if !required.is_empty()
        && let Some(object) = schema.as_object_mut()
    {
        object.insert("required".to_string(), json!(required));
    }

    schema
}

/// Human-readable description for the generated tool.
fn operation_description(operation: &OperationTool) -> String {
    let code = operation
        .definition
        .code
        .value
        .as_deref()
        .unwrap_or_default();

    let base = operation
        .definition
        .description
        .as_ref()
        .and_then(|d| d.value.as_deref())
        .map(str::to_string)
        .unwrap_or_else(|| format!("Invoke the FHIR ${code} operation.",));

    let resource_types = operation.resource_types();
    let mut detail = format!("{base} (FHIR operation ${code}");

    if !resource_types.is_empty() {
        let names: Vec<&str> = resource_types.iter().map(|r| r.as_ref()).collect();
        detail.push_str(&format!(" on {}", names.join(", ")));
    }

    if operation.affects_state() {
        detail.push_str("; modifies server state");
    }

    detail.push(')');
    detail
}

/// Build the MCP `Tool` describing an operation.
pub fn operation_to_tool(operation: &OperationTool) -> Tool {
    let title = operation
        .definition
        .title
        .as_ref()
        .and_then(|t| t.value.clone())
        .or_else(|| operation.definition.name.value.clone())
        .unwrap_or_else(|| operation.tool_name.clone());

    Tool {
        annotations: None,
        description: Some(operation_description(operation)),
        input_schema: operation_input_schema(operation),
        meta: None,
        name: operation.tool_name.clone(),
        output_schema: Some(json!({
            "type": "object",
            "description": "The operation result. Operations return either a FHIR Parameters \
                            resource or the resource named by the operation's output definition.",
        })),
        title: Some(title),
    }
}

/// Discover every operation invocable in the caller's tenant/project.
///
/// This searches `OperationDefinition`, which covers the server's built-in
/// operations (loaded from embedded artifacts) and any the user has created.
pub async fn discover_operations<
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>(
    ctx: Arc<ServerCTX<Client>>,
) -> Result<Vec<OperationTool>, MCPError<serde_json::Value>> {
    let bundle = ctx
        .client
        .search_type(
            ctx.clone(),
            ResourceType::OperationDefinition,
            // Operation codes are unique per project; pull the full set so the
            // tool list reflects everything the caller can actually invoke.
            vec![("_count".to_string(), vec!["1000".to_string()])].into(),
        )
        .await?;

    let mut operations: Vec<OperationTool> = Vec::new();

    for entry in bundle.entry.as_deref().unwrap_or_default() {
        let Some(Resource::OperationDefinition(definition)) = entry.resource.as_deref() else {
            continue;
        };

        let Some(code) = definition.code.value.as_deref() else {
            continue;
        };

        let tool_name = operation_tool_name(code);

        // Two definitions can collide once codes are sanitized into tool names;
        // keep the first so the advertised list stays unambiguous.
        if operations.iter().any(|o| o.tool_name == tool_name) {
            continue;
        }

        operations.push(OperationTool {
            definition: definition.clone(),
            tool_name,
        });
    }

    Ok(operations)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn definition_from_json(value: serde_json::Value) -> OperationDefinition {
        serde_json::from_value(value).expect("valid OperationDefinition")
    }

    fn tool_from_json(value: serde_json::Value) -> OperationTool {
        let definition = definition_from_json(value);
        let tool_name = operation_tool_name(definition.code.value.as_deref().unwrap_or_default());
        OperationTool {
            definition,
            tool_name,
        }
    }

    #[test]
    fn tool_names_are_sanitized() {
        assert_eq!(
            operation_tool_name("viewdefinition-run"),
            "fhir_op_viewdefinition_run"
        );
        assert_eq!(operation_tool_name("expand"), "fhir_op_expand");
        // User-authored codes may carry dots or mixed case.
        assert_eq!(
            operation_tool_name("myorg.riskScore"),
            "fhir_op_myorg_riskscore"
        );
    }

    #[test]
    fn input_schema_maps_types_and_cardinality() {
        let tool = tool_from_json(serde_json::json!({
            "resourceType": "OperationDefinition",
            "name": "Example",
            "status": "active",
            "kind": "operation",
            "code": "example",
            "system": true,
            "type": false,
            "instance": false,
            "parameter": [
                { "name": "count", "use": "in", "min": 1, "max": "1", "type": "integer" },
                { "name": "flag", "use": "in", "min": 0, "max": "1", "type": "boolean" },
                { "name": "tag", "use": "in", "min": 0, "max": "*", "type": "string" },
                { "name": "subject", "use": "in", "min": 0, "max": "1", "type": "Reference" },
                { "name": "return", "use": "out", "min": 1, "max": "1", "type": "Bundle" }
            ]
        }));

        let schema = operation_input_schema(&tool);
        let properties = schema["properties"].as_object().expect("properties");

        assert_eq!(properties["count"]["type"], "integer");
        assert_eq!(properties["flag"]["type"], "boolean");
        // Repeating parameters become arrays.
        assert_eq!(properties["tag"]["type"], "array");
        assert_eq!(properties["tag"]["items"]["type"], "string");
        // Complex datatypes are objects.
        assert_eq!(properties["subject"]["type"], "object");
        // Output parameters are not inputs.
        assert!(!properties.contains_key("return"));
        // min > 0 makes a parameter required.
        assert_eq!(schema["required"], serde_json::json!(["count"]));
    }

    #[test]
    fn instance_only_operations_require_an_id() {
        let tool = tool_from_json(serde_json::json!({
            "resourceType": "OperationDefinition",
            "name": "Everything",
            "status": "active",
            "kind": "operation",
            "code": "everything",
            "resource": ["Patient"],
            "system": false,
            "type": false,
            "instance": true
        }));

        let schema = operation_input_schema(&tool);
        assert_eq!(schema["properties"]["id"]["type"], "string");
        assert_eq!(schema["required"], serde_json::json!(["id"]));
        // A single resource type needs no disambiguation.
        assert!(
            !schema["properties"]
                .as_object()
                .unwrap()
                .contains_key("resourceType")
        );
    }

    #[test]
    fn id_is_optional_when_other_levels_are_supported() {
        let tool = tool_from_json(serde_json::json!({
            "resourceType": "OperationDefinition",
            "name": "ViewDefinitionRun",
            "status": "active",
            "kind": "operation",
            "code": "viewdefinition-run",
            "resource": ["ViewDefinition"],
            "system": true,
            "type": true,
            "instance": true
        }));

        let schema = operation_input_schema(&tool);
        assert_eq!(schema["properties"]["id"]["type"], "string");
        // id must not be required when type/system invocation is also allowed.
        assert!(schema.get("required").is_none());
        assert_eq!(
            tool.levels(),
            vec![
                OperationLevel::Instance,
                OperationLevel::Type,
                OperationLevel::System
            ]
        );
    }

    #[test]
    fn multiple_resource_types_are_disambiguated() {
        let tool = tool_from_json(serde_json::json!({
            "resourceType": "OperationDefinition",
            "name": "Validate",
            "status": "active",
            "kind": "operation",
            "code": "validate",
            "resource": ["Patient", "Observation"],
            "system": false,
            "type": true,
            "instance": false
        }));

        let schema = operation_input_schema(&tool);
        assert_eq!(
            schema["properties"]["resourceType"]["enum"],
            serde_json::json!(["Patient", "Observation"])
        );
    }

    #[test]
    fn affects_state_defaults_to_false() {
        let read_only = tool_from_json(serde_json::json!({
            "resourceType": "OperationDefinition",
            "name": "Example",
            "status": "active",
            "kind": "operation",
            "code": "example",
            "system": true,
            "type": false,
            "instance": false
        }));
        assert!(!read_only.affects_state());

        let mutating = tool_from_json(serde_json::json!({
            "resourceType": "OperationDefinition",
            "name": "Example",
            "status": "active",
            "kind": "operation",
            "code": "example",
            "affectsState": true,
            "system": true,
            "type": false,
            "instance": false
        }));
        assert!(mutating.affects_state());
    }
}
