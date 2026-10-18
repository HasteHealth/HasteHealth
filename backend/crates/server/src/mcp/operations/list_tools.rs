use crate::{
    fhir_client::ServerCTX,
    mcp::{
        error::MCPError,
        operations::{
            operation_tools::{discover_operations, operation_to_tool},
            scopes::{allows_any, allows_operation},
        },
        request::ListToolsRequest,
        schemas::types::{ListToolsResult, Tool},
    },
};
use haste_fhir_client::FHIRClient;
use haste_fhir_model::r4::generated::{
    resources::CapabilityStatementRestResourceSearchParam,
    terminology::{BoundCode, SearchParamType},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::scopes::{Scopes, SmartResourceScopePermission};
use serde_json::json;
use std::sync::Arc;

pub const R4_SEARCH_TOOL_NAME: &str = "fhir_r4_search";
pub const GET_SEARCH_PARAMETERS_TOOL_NAME: &str = "fhir_r4_get_search_parameters";
pub const GET_RESOURCE_SCHEMA_TOOL_NAME: &str = "fhir_r4_get_resource_schema";
pub const R4_READ_TOOL_NAME: &str = "fhir_r4_read";
pub const R4_VREAD_TOOL_NAME: &str = "fhir_r4_vread";
pub const R4_CREATE_TOOL_NAME: &str = "fhir_r4_create";
pub const R4_UPDATE_TOOL_NAME: &str = "fhir_r4_update";
pub const R4_PATCH_TOOL_NAME: &str = "fhir_r4_patch";
pub const R4_DELETE_TOOL_NAME: &str = "fhir_r4_delete";
pub const R4_HISTORY_INSTANCE_TOOL_NAME: &str = "fhir_r4_history_instance";
pub const R4_HISTORY_TYPE_TOOL_NAME: &str = "fhir_r4_history_type";
pub const R4_CAPABILITIES_TOOL_NAME: &str = "fhir_r4_capabilities";
pub const R4_TRANSACTION_TOOL_NAME: &str = "fhir_r4_transaction";
pub const R4_BATCH_TOOL_NAME: &str = "fhir_r4_batch";

/// Returns the base URL for external JSON Schema `$ref`s.
/// Points to the `/schemas/fhir/{ResourceType}` endpoint so MCP tool
/// input/output schemas use external refs instead of inlining definitions.
pub fn schema_base_url(api_uri: &str) -> String {
    format!("{}/schemas/fhir", api_uri)
}

/// How a value of one search parameter type is written: its syntax, an
/// example and the modifiers the search engine accepts for it.
struct SearchValueSyntax {
    syntax: &'static str,
    example: &'static str,
    modifiers: &'static [&'static str],
}

/// Syntax for a parameter type, or `None` for the composite and special types
/// the search engine does not index.
fn search_value_syntax(type_: &BoundCode<SearchParamType>) -> Option<SearchValueSyntax> {
    Some(match type_.as_str()? {
        "number" => SearchValueSyntax {
            syntax: "number with an optional prefix eq, ne, gt, lt, ge, le, sa, eb or ap",
            example: "gt5",
            modifiers: &[":missing"],
        },
        "date" => SearchValueSyntax {
            syntax: "date or dateTime (YYYY, YYYY-MM-DD or full instant) with an optional \
                     prefix eq, ne, gt, lt, ge, le, sa, eb or ap; a partial date covers its \
                     whole period",
            example: "ge2024-01-01",
            modifiers: &[":missing"],
        },
        "quantity" => SearchValueSyntax {
            syntax: "number[|system|code] with an optional prefix eq, ne, gt, lt, ge, le, sa, \
                     eb or ap",
            example: "gt5.4|http://unitsofmeasure.org|mg",
            modifiers: &[":missing"],
        },
        "string" => SearchValueSyntax {
            syntax: "text matched from the start of each word, case-insensitive; :exact for the \
                     whole value, :contains for anywhere in it",
            example: "Smith",
            modifiers: &[":exact", ":contains", ":text", ":missing"],
        },
        "token" => SearchValueSyntax {
            syntax: "system|code, code alone (any system) or |code (no system); :text matches \
                     the display, :not excludes, :of-type matches identifiers by type",
            example: "http://loinc.org|8867-4",
            modifiers: &[
                ":not", ":text", ":in", ":not-in", ":above", ":below", ":of-type", ":missing",
            ],
        },
        "reference" => SearchValueSyntax {
            syntax: "Type/id, an id alone or a full URL; narrow the target type with :Type \
                     (subject:Patient) and chain into the target's parameters with a dot \
                     (subject.name=Smith)",
            example: "Patient/123",
            modifiers: &[":Type", ":identifier", ":above", ":below", ":missing"],
        },
        "uri" => SearchValueSyntax {
            syntax: "the URI exactly; :below for everything under it, :above for its parents",
            example: "http://hl7.org/fhir/StructureDefinition/Patient",
            modifiers: &[":above", ":below", ":missing"],
        },
        _ => return None,
    })
}

pub fn search_tool_parameters(
    capability_search_params: &Vec<CapabilityStatementRestResourceSearchParam>,
) -> serde_json::Value {
    let mut properties: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();

    for capability_parameter in capability_search_params.iter() {
        let name = capability_parameter.name.value.clone().unwrap_or_default();
        let description = capability_parameter
            .documentation
            .as_ref()
            .and_then(|d| d.value.as_ref());

        // Every value is written as a string: a number or date carries its
        // comparison prefix in the text (gt5, ge2024-01-01).
        let Some(value_syntax) = search_value_syntax(&capability_parameter.type_) else {
            continue;
        };

        // `_id` is a token in the definitions but holds a plain logical id.
        let (syntax, example) = if name == "_id" {
            ("the resource's logical id", "abc123")
        } else {
            (value_syntax.syntax, value_syntax.example)
        };

        properties.insert(
            name,
            json!({
                "type": "string",
                "description": description,
                "fhirType": capability_parameter.type_.as_str(),
                "syntax": syntax,
                "example": example,
                "modifiers": value_syntax.modifiers,
            }),
        );
    }

    json!({
        "type": "object",
        "description": "Search parameters by name. Each gives its FHIR type, how a value is \
                        written, an example and the modifiers it accepts (appended to the name, \
                        as in name:exact). Comma-separated values match any of them.",
        "properties": serde_json::Value::Object(properties),
    })
}

/// Schema of the `resourceType` argument the per-type tools share.
///
/// A plain string rather than an enum of every supported type: the enum ran to
/// 155 values and, repeated on 11 tools, was close to half of the tool list.
/// The server validates the type on each call and suggests the closest name on
/// a miss; the capabilities tool lists the full set.
fn resource_type_schema() -> serde_json::Value {
    json!({
        "type": "string",
        "description": format!(
            "FHIR resource type, e.g. Patient or Observation (case-sensitive). \
             The '{}' tool lists every type this project supports.",
            R4_CAPABILITIES_TOOL_NAME
        ),
    })
}

fn generate_search_schema() -> Tool {
    let input_schema = json!({
      "type": "object",
      "properties": {
        "resourceType": resource_type_schema(),
        "search_parameters": {
          "type": "object",
          "description": format!(
            "Search parameters keyed by name, as in a FHIR query string; the '{}' tool lists \
             the names for a resource type. Values use FHIR search syntax: tokens as system|code, \
             dates, numbers and quantities with a prefix (ge2024-01-01, gt5), references as \
             Type/id. A comma between values matches any of them (OR); an array repeats the \
             parameter so every value must match (AND), e.g. \
             {{\"date\": [\"ge2024-01-01\", \"lt2025-01-01\"]}}. Modifiers go on the name \
             (name:exact, subject:Patient, identifier:missing) and chains use dots \
             (subject.name). Result parameters: _count and _offset to page; _sort (-name for \
             descending); _include and _revinclude (Observation:subject); _elements (comma-separated \
             element names) or _summary=true to shrink each resource; _total=accurate or estimate.",
            GET_SEARCH_PARAMETERS_TOOL_NAME),
          "additionalProperties": {
            "oneOf": [
              { "type": "string" },
              { "type": "array", "items": { "type": "string" } },
            ],
          },
        },
      },
      "required": ["resourceType"]
    });

    Tool {
        annotations: None,
        description: Some("Tool for FHIR Resource Search across supported types".to_string()),

        input_schema,
        meta: None,
        name: R4_SEARCH_TOOL_NAME.to_string(),
        output_schema: Some(haste_sd_to_json_schema::bundle_of_resource(&json!({
            "type": "object"
        }))),
        title: Some(R4_SEARCH_TOOL_NAME.to_string()),
    }
}

fn generate_get_search_parameters_tool() -> Tool {
    let input_schema = json!({
      "type": "object",
      "properties": {
        "resourceType": resource_type_schema(),
      },
      "required": ["resourceType"]
    });

    Tool {
        annotations: None,
        description: Some(
            "Tool to get available search parameters for a given FHIR Resource Type".to_string(),
        ),

        input_schema,
        meta: None,
        name: GET_SEARCH_PARAMETERS_TOOL_NAME.to_string(),
        output_schema: Some(json!({
            "type": "object",
            "description": "JSON Schema describing the available search parameters for the specified FHIR Resource Type",
        })),
        title: Some(GET_SEARCH_PARAMETERS_TOOL_NAME.to_string()),
    }
}

fn generate_get_resource_schema_tool() -> Tool {
    let input_schema = json!({
      "type": "object",
      "properties": {
        "resourceType": resource_type_schema(),
      },
      "required": ["resourceType"]
    });

    Tool {
        annotations: None,
        description: Some(
            "Tool to get the exact JSON Schema (Draft 2020-12) for a given FHIR Resource Type. \
             Use this to see the full field structure before creating or updating a resource, \
             or to interpret the shape of a resource returned by another tool."
                .to_string(),
        ),

        input_schema,
        meta: None,
        name: GET_RESOURCE_SCHEMA_TOOL_NAME.to_string(),
        output_schema: Some(json!({
            "type": "object",
            "description": "JSON Schema describing the structure of the specified FHIR Resource Type",
        })),
        title: Some(GET_RESOURCE_SCHEMA_TOOL_NAME.to_string()),
    }
}

fn generate_read_tool() -> Tool {
    Tool {
        annotations: None,
        description: Some(
            "Read a specific FHIR resource by its resource type and logical ID".to_string(),
        ),

        input_schema: json!({
            "type": "object",
            "properties": {
                "resourceType": resource_type_schema(),
                "id": {
                    "type": "string",
                    "description": "The logical ID of the resource to read",
                },
            },
            "required": ["resourceType", "id"]
        }),
        meta: None,
        name: R4_READ_TOOL_NAME.to_string(),
        output_schema: Some(json!({
            "type": "object",
            "description": format!(
                "The requested FHIR resource. Shape depends on the resourceType - use the '{}' tool to get its exact schema.",
                GET_RESOURCE_SCHEMA_TOOL_NAME
            ),
        })),
        title: Some("Read FHIR Resource".to_string()),
    }
}

fn generate_vread_tool() -> Tool {
    Tool {
        annotations: None,
        description: Some(
            "Read a specific version of a FHIR resource by resource type, logical ID, and version ID"
                .to_string(),
        ),

        input_schema: json!({
            "type": "object",
            "properties": {
                "resourceType": resource_type_schema(),
                "id": {
                    "type": "string",
                    "description": "The logical ID of the resource",
                },
                "versionId": {
                    "type": "string",
                    "description": "The version ID to read",
                },
            },
            "required": ["resourceType", "id", "versionId"]
        }),
        meta: None,
        name: R4_VREAD_TOOL_NAME.to_string(),
        output_schema: Some(json!({
            "type": "object",
            "description": format!(
                "The requested version of the FHIR resource. Shape depends on the resourceType - use the '{}' tool to get its exact schema.",
                GET_RESOURCE_SCHEMA_TOOL_NAME
            ),
        })),
        title: Some("Version Read FHIR Resource".to_string()),
    }
}

fn generate_create_tool() -> Tool {
    Tool {
        annotations: None,
        description: Some(
            "Create a new FHIR resource. Provide the resourceType and the full resource body."
                .to_string(),
        ),

        input_schema: json!({
            "type": "object",
            "properties": {
                "resourceType": resource_type_schema(),
                "resource": {
                    "type": "object",
                    "description": "The FHIR resource to create. Must include resourceType field matching the resourceType parameter.",
                },
            },
            "required": ["resourceType", "resource"]
        }),
        meta: None,
        name: R4_CREATE_TOOL_NAME.to_string(),
        output_schema: Some(json!({
            "type": "object",
            "description": format!(
                "The created FHIR resource with server-assigned ID. Shape depends on the resourceType - use the '{}' tool to get its exact schema.",
                GET_RESOURCE_SCHEMA_TOOL_NAME
            ),
        })),
        title: Some("Create FHIR Resource".to_string()),
    }
}

fn generate_update_tool() -> Tool {
    Tool {
        annotations: None,
        description: Some(
            "Update an existing FHIR resource by its resource type and logical ID. Replaces the entire resource."
                .to_string(),
        ),

        input_schema: json!({
            "type": "object",
            "properties": {
                "resourceType": resource_type_schema(),
                "id": {
                    "type": "string",
                    "description": "The logical ID of the resource to update",
                },
                "resource": {
                    "type": "object",
                    "description": "The complete FHIR resource to replace the existing one. Must include resourceType and id fields.",
                },
            },
            "required": ["resourceType", "id", "resource"]
        }),
        meta: None,
        name: R4_UPDATE_TOOL_NAME.to_string(),
        output_schema: Some(json!({
            "type": "object",
            "description": format!(
                "The updated FHIR resource. Shape depends on the resourceType - use the '{}' tool to get its exact schema.",
                GET_RESOURCE_SCHEMA_TOOL_NAME
            ),
        })),
        title: Some("Update FHIR Resource".to_string()),
    }
}

fn generate_patch_tool() -> Tool {
    Tool {
        annotations: None,
        description: Some(
            "Partially update a FHIR resource using JSON Patch (RFC 6902). Provide an array of patch operations."
                .to_string(),
        ),

        input_schema: json!({
            "type": "object",
            "properties": {
                "resourceType": resource_type_schema(),
                "id": {
                    "type": "string",
                    "description": "The logical ID of the resource to patch",
                },
                "patches": {
                    "type": "array",
                    "description": "JSON Patch (RFC 6902) operations array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "op": {
                                "type": "string",
                                "enum": ["add", "remove", "replace", "move", "copy", "test"],
                            },
                            "path": {
                                "type": "string",
                                "description": "JSON Pointer (RFC 6901) path to the target location",
                            },
                            "value": {
                                "description": "The value to apply (required for add, replace, test)",
                            },
                            "from": {
                                "type": "string",
                                "description": "Source path for move/copy operations",
                            },
                        },
                        "required": ["op", "path"],
                    },
                },
            },
            "required": ["resourceType", "id", "patches"]
        }),
        meta: None,
        name: R4_PATCH_TOOL_NAME.to_string(),
        output_schema: Some(json!({
            "type": "object",
            "description": format!(
                "The patched FHIR resource. Shape depends on the resourceType - use the '{}' tool to get its exact schema.",
                GET_RESOURCE_SCHEMA_TOOL_NAME
            ),
        })),
        title: Some("Patch FHIR Resource".to_string()),
    }
}

fn generate_delete_tool() -> Tool {
    Tool {
        annotations: None,
        description: Some("Delete a FHIR resource by its resource type and logical ID".to_string()),

        input_schema: json!({
            "type": "object",
            "properties": {
                "resourceType": resource_type_schema(),
                "id": {
                    "type": "string",
                    "description": "The logical ID of the resource to delete",
                },
            },
            "required": ["resourceType", "id"]
        }),
        meta: None,
        name: R4_DELETE_TOOL_NAME.to_string(),
        output_schema: Some(json!({
            "type": "object",
            "properties": {
                "success": { "type": "boolean" },
                "message": { "type": "string" },
            },
        })),
        title: Some("Delete FHIR Resource".to_string()),
    }
}

fn generate_history_instance_tool() -> Tool {
    Tool {
        annotations: None,
        description: Some(
            "Retrieve the version history for a specific FHIR resource instance".to_string(),
        ),

        input_schema: json!({
            "type": "object",
            "properties": {
                "resourceType": resource_type_schema(),
                "id": {
                    "type": "string",
                    "description": "The logical ID of the resource",
                },
                "_count": {
                    "type": "string",
                    "description": "Maximum number of history entries to return",
                },
                "_since": {
                    "type": "string",
                    "description": "Only include history entries after this instant (ISO 8601)",
                },
            },
            "required": ["resourceType", "id"]
        }),
        meta: None,
        name: R4_HISTORY_INSTANCE_TOOL_NAME.to_string(),
        output_schema: Some(haste_sd_to_json_schema::bundle_of_resource(&json!({
            "type": "object"
        }))),
        title: Some("Instance History".to_string()),
    }
}

fn generate_history_type_tool() -> Tool {
    Tool {
        annotations: None,
        description: Some(
            "Retrieve the version history for all resources of a given type".to_string(),
        ),

        input_schema: json!({
            "type": "object",
            "properties": {
                "resourceType": resource_type_schema(),
                "_count": {
                    "type": "string",
                    "description": "Maximum number of history entries to return",
                },
                "_since": {
                    "type": "string",
                    "description": "Only include history entries after this instant (ISO 8601)",
                },
            },
            "required": ["resourceType"]
        }),
        meta: None,
        name: R4_HISTORY_TYPE_TOOL_NAME.to_string(),
        output_schema: Some(haste_sd_to_json_schema::bundle_of_resource(&json!({
            "type": "object"
        }))),
        title: Some("Type History".to_string()),
    }
}

fn generate_capabilities_tool() -> Tool {
    Tool {
        annotations: None,
        description: Some(
            "Retrieve the server's CapabilityStatement describing supported resources, operations, and search parameters"
                .to_string(),
        ),

        input_schema: json!({
            "type": "object",
            "properties": {},
        }),
        meta: None,
        name: R4_CAPABILITIES_TOOL_NAME.to_string(),
        output_schema: Some(json!({
            "type": "object",
            "description": "FHIR CapabilityStatement resource describing server capabilities",
        })),
        title: Some("Server Capabilities".to_string()),
    }
}

fn generate_transaction_tool(api_uri: &str) -> Tool {
    let base = schema_base_url(api_uri);

    Tool {
        annotations: None,
        description: Some(
            "Execute a FHIR transaction Bundle. All entries are processed atomically — if any entry fails, the entire transaction is rolled back."
                .to_string(),
        ),

        input_schema: json!({
            "type": "object",
            "properties": {
                "bundle": {
                    "$ref": format!("{}/Bundle", base),
                    "description": "A FHIR Bundle with type 'transaction'. Each entry must have a request element with method and url.",
                },
            },
            "required": ["bundle"]
        }),
        meta: None,
        name: R4_TRANSACTION_TOOL_NAME.to_string(),
        output_schema: Some(json!({
            "type": "object",
            "$ref": format!("{}/Bundle", base),
            "description": "Transaction response Bundle with one entry per request, containing status and resource outcomes.",
        })),
        title: Some("FHIR Transaction".to_string()),
    }
}

fn generate_batch_tool(api_uri: &str) -> Tool {
    let base = schema_base_url(api_uri);

    Tool {
        annotations: None,
        description: Some(
            "Execute a FHIR batch Bundle. Each entry is processed independently — failures in one entry do not affect others."
                .to_string(),
        ),

        input_schema: json!({
            "type": "object",
            "properties": {
                "bundle": {
                    "$ref": format!("{}/Bundle", base),
                    "description": "A FHIR Bundle with type 'batch'. Each entry must have a request element with method and url.",
                },
            },
            "required": ["bundle"]
        }),
        meta: None,
        name: R4_BATCH_TOOL_NAME.to_string(),
        output_schema: Some(json!({
            "type": "object",
            "$ref": format!("{}/Bundle", base),
            "description": "Batch response Bundle with one entry per request, containing individual status codes and outcomes.",
        })),
        title: Some("FHIR Batch".to_string()),
    }
}

/// Permission a static tool requires, used to decide whether to advertise it.
///
/// Tools that are not tied to one permission (schema lookups, capability
/// statement, bundles) are always listed: they either describe the server
/// rather than touch data, or -- in the case of bundles -- carry requests whose
/// individual entries are scope-checked as they are processed.
fn static_tool_permission(tool_name: &str) -> Option<SmartResourceScopePermission> {
    match tool_name {
        R4_SEARCH_TOOL_NAME | R4_HISTORY_INSTANCE_TOOL_NAME | R4_HISTORY_TYPE_TOOL_NAME => {
            Some(SmartResourceScopePermission::Search)
        }
        R4_READ_TOOL_NAME | R4_VREAD_TOOL_NAME => Some(SmartResourceScopePermission::Read),
        R4_CREATE_TOOL_NAME => Some(SmartResourceScopePermission::Create),
        R4_UPDATE_TOOL_NAME | R4_PATCH_TOOL_NAME => Some(SmartResourceScopePermission::Update),
        R4_DELETE_TOOL_NAME => Some(SmartResourceScopePermission::Delete),
        _ => None,
    }
}

/// Drop tools the caller's SMART scopes could never permit.
fn filter_by_scopes(tools: Vec<Tool>, scopes: &Scopes) -> Vec<Tool> {
    tools
        .into_iter()
        .filter(|tool| match static_tool_permission(&tool.name) {
            Some(permission) => allows_any(scopes, &permission),
            None => true,
        })
        .collect()
}

pub async fn list_tools<
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>(
    ctx: Arc<ServerCTX<Client>>,
    _request: &ListToolsRequest,
    api_uri: &str,
) -> Result<ListToolsResult, MCPError<serde_json::Value>> {
    let scopes = &ctx.user.claims.scope;

    let static_tools = vec![
        generate_search_schema(),
        generate_get_search_parameters_tool(),
        generate_get_resource_schema_tool(),
        generate_read_tool(),
        generate_vread_tool(),
        generate_create_tool(),
        generate_update_tool(),
        generate_patch_tool(),
        generate_delete_tool(),
        generate_history_instance_tool(),
        generate_history_type_tool(),
        generate_capabilities_tool(),
        generate_transaction_tool(api_uri),
        generate_batch_tool(api_uri),
    ];

    let mut tools = filter_by_scopes(static_tools, scopes);

    // Every operation the caller can invoke -- the server's built-in ones and any
    // the project has defined -- is advertised as its own tool.
    let operations = discover_operations(ctx.clone()).await?;
    tools.extend(
        operations
            .iter()
            .filter(|operation| allows_operation(scopes, operation))
            .map(operation_to_tool),
    );

    Ok(ListToolsResult {
        tools,
        meta: None,
        next_cursor: None,
    })
}
