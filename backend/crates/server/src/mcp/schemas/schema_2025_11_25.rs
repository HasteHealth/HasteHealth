//! Hand-written MCP protocol types for the 2025-11-25 specification.
//!
//! These replace the previously auto-generated types from the JSON schema,
//! keeping only the types actually used by the server implementation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// RequestId
// ---------------------------------------------------------------------------

/// A JSON-RPC request identifier — either a string or an integer.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum RequestId {
    String(String),
    Integer(i64),
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => f.write_str(s),
            Self::Integer(n) => write!(f, "{n}"),
        }
    }
}

impl From<i64> for RequestId {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<String> for RequestId {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

// ---------------------------------------------------------------------------
// Annotations
// ---------------------------------------------------------------------------

/// Optional annotations for the client, used to inform how objects are
/// displayed or prioritised.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Annotations {
    /// Describes who the intended audience of this data is.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub audience: Vec<String>,

    /// ISO 8601 timestamp of when the resource was last modified.
    #[serde(
        rename = "lastModified",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub last_modified: Option<String>,

    /// Importance from 0.0 (least) to 1.0 (most).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<f64>,
}

// ---------------------------------------------------------------------------
// Content types
// ---------------------------------------------------------------------------

/// Text content in a tool result.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TextContent {
    /// Optional annotations for the client.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,

    /// Protocol metadata.
    #[serde(rename = "_meta", default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<TextContentMeta>,

    /// The text content of the message.
    pub text: String,

    /// Content type discriminator — always `"text"`.
    #[serde(rename = "type")]
    pub type_: String,
}

/// Metadata for [`TextContent`]. Currently empty per the MCP spec.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct TextContentMeta {}

/// A content block in an MCP tool result.
///
/// Only the variants actually used by this server are defined.
/// Additional variants (ImageContent, AudioContent, ResourceLink,
/// EmbeddedResource) can be added when needed.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    TextContent(TextContent),
}

impl From<TextContent> for ContentBlock {
    fn from(value: TextContent) -> Self {
        Self::TextContent(value)
    }
}

// ---------------------------------------------------------------------------
// Tool
// ---------------------------------------------------------------------------

/// Describes a tool that the server exposes.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Tool {
    /// Optional behavioural annotations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: Option<ToolAnnotations>,

    /// Human-readable description used as an LLM hint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Execution-related properties.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution: Option<ToolExecution>,

    /// Optional icons for UI display.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub icons: Vec<Icon>,

    /// JSON Schema describing the tool's input parameters.
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,

    /// Protocol metadata.
    #[serde(rename = "_meta", default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,

    /// Programmatic identifier for the tool.
    pub name: String,

    /// Optional JSON Schema describing the tool's structured output.
    #[serde(
        rename = "outputSchema",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub output_schema: Option<serde_json::Value>,

    /// Human-readable display name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Behavioural hints for a [`Tool`].
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ToolAnnotations {
    #[serde(
        rename = "destructiveHint",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub destructive_hint: Option<bool>,

    #[serde(
        rename = "idempotentHint",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub idempotent_hint: Option<bool>,

    #[serde(
        rename = "openWorldHint",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub open_world_hint: Option<bool>,

    #[serde(
        rename = "readOnlyHint",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub read_only_hint: Option<bool>,

    /// Human-readable title for the tool annotation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Execution-related properties for a tool.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ToolExecution {
    /// Estimated execution time in seconds.
    #[serde(
        rename = "estimatedSeconds",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub estimated_seconds: Option<f64>,
}

/// An icon for UI display.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Icon {
    /// URI of the icon image.
    pub uri: String,

    /// MIME type of the icon (e.g. `image/png`).
    #[serde(rename = "mimeType", default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,

    /// Intended size in pixels (width × height).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<u32>,
}

// ---------------------------------------------------------------------------
// Tool call
// ---------------------------------------------------------------------------

/// Parameters for a `tools/call` request.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CallToolRequestParams {
    /// Arguments to pass to the tool (opaque JSON object).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arguments: Option<serde_json::Value>,

    /// Protocol metadata.
    #[serde(rename = "_meta", default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,

    /// The name of the tool to call.
    pub name: String,

    /// Optional task metadata for task-augmented execution.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task: Option<serde_json::Value>,
}

/// The server's response to a `tools/call` request.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CallToolResult {
    /// Unstructured content blocks representing the tool result.
    pub content: Vec<ContentBlock>,

    /// Whether the tool call ended in an error.
    #[serde(
        rename = "isError",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub is_error: Option<bool>,

    /// Protocol metadata.
    #[serde(rename = "_meta", default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,

    /// Optional structured (JSON) result of the tool call.
    #[serde(
        rename = "structuredContent",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub structured_content: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// List tools
// ---------------------------------------------------------------------------

/// Result of a `tools/list` request.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ListToolsResult {
    /// Protocol metadata.
    #[serde(rename = "_meta", default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,

    /// Pagination cursor for the next page of results, if any.
    #[serde(
        rename = "nextCursor",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub next_cursor: Option<String>,

    /// The tools available on this server.
    pub tools: Vec<Tool>,
}

// ---------------------------------------------------------------------------
// Initialize
// ---------------------------------------------------------------------------

/// Parameters sent by the client in an `initialize` request.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InitializeRequestParams {
    /// Client capabilities.
    pub capabilities: serde_json::Value,

    /// Information about the connecting client.
    #[serde(rename = "clientInfo")]
    pub client_info: serde_json::Value,

    /// Protocol metadata.
    #[serde(rename = "_meta", default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,

    /// The MCP protocol version the client supports.
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
}

/// Describes the server (or client) implementation.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Implementation {
    /// Human-readable description of this implementation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Optional icons for UI display.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub icons: Vec<Icon>,

    /// Programmatic name / identifier.
    pub name: String,

    /// Human-readable display title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Version string.
    pub version: String,

    /// Optional URL of the implementation's website.
    #[serde(
        rename = "websiteUrl",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub website_url: Option<String>,
}

/// The server's response to an `initialize` request.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InitializeResult {
    /// Server capabilities.
    pub capabilities: ServerCapabilities,

    /// Optional instructions / hints for the LLM.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,

    /// Protocol metadata.
    #[serde(rename = "_meta", default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,

    /// The MCP protocol version the server wants to use.
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,

    /// Information about the server implementation.
    #[serde(rename = "serverInfo")]
    pub server_info: Implementation,
}

// ---------------------------------------------------------------------------
// Server capabilities
// ---------------------------------------------------------------------------

/// Capabilities a server may advertise.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ServerCapabilities {
    /// Argument autocompletion support.
    #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
    pub completions: serde_json::Map<String, serde_json::Value>,

    /// Experimental / non-standard capabilities.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub experimental: HashMap<String, serde_json::Map<String, serde_json::Value>>,

    /// Log message support.
    #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
    pub logging: serde_json::Map<String, serde_json::Value>,

    /// Prompt template support.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompts: Option<serde_json::Value>,

    /// Resource support.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resources: Option<serde_json::Value>,

    /// Task support.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tasks: Option<serde_json::Value>,

    /// Tool support.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<ServerCapabilitiesTools>,
}

/// Tool-specific capability flags.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ServerCapabilitiesTools {
    /// Whether the server supports notifications for changes to the tool list.
    #[serde(
        rename = "listChanged",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub list_changed: Option<bool>,
}

// ---------------------------------------------------------------------------
// ServerResult — the union type used to wrap all possible server responses
// ---------------------------------------------------------------------------

/// A union wrapper that serialises whichever result variant is populated.
///
/// The generated code used numbered `subtype_N` fields with `#[serde(flatten)]`.
/// We preserve the exact same field names and flatten behaviour so that
/// existing application code (`route.rs`) continues to work unchanged.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ServerResult {
    #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
    pub subtype_0: Option<serde_json::Value>,

    #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
    pub subtype_1: Option<InitializeResult>,

    #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
    pub subtype_2: Option<serde_json::Value>,

    #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
    pub subtype_3: Option<serde_json::Value>,

    #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
    pub subtype_4: Option<serde_json::Value>,

    #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
    pub subtype_5: Option<serde_json::Value>,

    #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
    pub subtype_6: Option<serde_json::Value>,

    #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
    pub subtype_7: Option<ListToolsResult>,

    #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
    pub subtype_8: Option<CallToolResult>,

    #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
    pub subtype_9: Option<serde_json::Value>,

    #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
    pub subtype_10: Option<serde_json::Value>,

    #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
    pub subtype_11: Option<serde_json::Value>,

    #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
    pub subtype_12: Option<serde_json::Value>,

    #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
    pub subtype_13: Option<serde_json::Value>,
}
