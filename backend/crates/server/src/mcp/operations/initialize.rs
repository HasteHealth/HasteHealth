use crate::{
    fhir_client::ServerCTX,
    mcp::{
        error::MCPError,
        operations::{
            GET_RESOURCE_SCHEMA_TOOL_NAME, GET_SEARCH_PARAMETERS_TOOL_NAME, OPERATION_TOOL_PREFIX,
            R4_CAPABILITIES_TOOL_NAME, R4_CREATE_TOOL_NAME, R4_DELETE_TOOL_NAME,
            R4_PATCH_TOOL_NAME, R4_READ_TOOL_NAME, R4_SEARCH_TOOL_NAME, R4_TRANSACTION_TOOL_NAME,
            R4_UPDATE_TOOL_NAME,
        },
        request::InitializeRequest,
        schemas::types::{
            Implementation, InitializeResult, ServerCapabilities, ServerCapabilitiesTools,
        },
    },
};
use haste_fhir_client::FHIRClient;
use haste_fhir_operation_error::OperationOutcomeError;
use std::sync::Arc;

pub fn initialize<Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static>(
    _ctx: Arc<ServerCTX<Client>>,
    _request: &InitializeRequest,
) -> Result<InitializeResult, MCPError<serde_json::Value>> {
    Ok(InitializeResult {
        capabilities: ServerCapabilities {
            tools: Some(ServerCapabilitiesTools {
                list_changed: Some(false),
            }),
            ..ServerCapabilities::default()
        },
        instructions: Some(instructions()),
        meta: None,
        protocol_version: "2025-03-26".to_string(),
        server_info: Implementation {
            name: "Haste Health MCP Server".to_string(),
            version: "0.0.1".to_string(),
            title: Some("Haste Health MCP Server".to_string()),
        },
    })
}

/// Guidance a client adds to the model's context before its first tool call:
/// the order to work in, FHIR search syntax, and when to confirm with the user.
///
/// It names only tools every project has; the full set, including the
/// per-operation tools, comes from `tools/list`.
fn instructions() -> String {
    format!(
        "Haste Health FHIR R4 server for one project. Tools named fhir_r4_* are the FHIR REST API; \
         tools named {OPERATION_TOOL_PREFIX}* invoke FHIR operations such as $validate or $everything.\n\
         \n\
         How to work:\n\
         1. Call {R4_CAPABILITIES_TOOL_NAME} once to learn the resource types, interactions and \
         operations this project supports. Besides the FHIR R4 types it serves Haste Health types \
         such as User, Membership and ClientApplication.\n\
         2. Before the first search on a resource type, call {GET_SEARCH_PARAMETERS_TOOL_NAME} for it. \
         Search parameter names differ from element names (Patient 'birthdate', not 'birthDate').\n\
         3. Search with {R4_SEARCH_TOOL_NAME}. Values use FHIR search syntax: tokens as system|code, \
         dates and numbers with a prefix (ge2024-01-01, gt5), references as Type/id (Patient/123), \
         comma-separated values to match any of them. Page with _count and _offset, order with _sort \
         (-field for descending), and pull referenced resources with _include or _revinclude.\n\
         4. Keep responses small: ask for _elements=id,name,... or _summary=true and a small _count \
         when you need only a few fields.\n\
         5. Read one resource with {R4_READ_TOOL_NAME}, create with {R4_CREATE_TOOL_NAME}, replace \
         with {R4_UPDATE_TOOL_NAME}, change single elements with {R4_PATCH_TOOL_NAME} (JSON Patch), \
         and send several requests at once with {R4_TRANSACTION_TOOL_NAME}.\n\
         \n\
         Rules:\n\
         - Every resource body carries resourceType, and references are Type/id. Create a referenced \
         resource first, or send both in one transaction with urn:uuid fullUrls.\n\
         - Call {GET_RESOURCE_SCHEMA_TOOL_NAME} when unsure of a resource's structure. The server \
         validates every write and answers an OperationOutcome that names the problem.\n\
         - Writes are permanent. Confirm with the user before {R4_DELETE_TOOL_NAME} and before an \
         update that drops data the resource already holds.\n\
         - An error carries a FHIR OperationOutcome. Read its issues and fix the request rather than \
         retrying it unchanged."
    )
}
