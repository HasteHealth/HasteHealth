#![allow(dead_code)]
//! MCP (Model Context Protocol) types aligned with the 2025-06-18 specification.
//!
//! These are hand-written structs based on the official MCP JSON Schema at:
//! <https://github.com/modelcontextprotocol/modelcontextprotocol/tree/main/schema>
//!
//! Only types relevant to a server implementation are defined. Additional types
//! can be added as the server's MCP surface area grows.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ===========================================================================
// JSON-RPC Primitives
// ===========================================================================

/// A uniquely identifying ID for a JSON-RPC request — either a string or integer.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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

/// A progress token, used to associate progress notifications with their
/// originating request. Can be a string or integer.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum ProgressToken {
    String(String),
    Integer(i64),
}

/// A JSON-RPC error response.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JsonRpcError {
    pub id: RequestId,
    pub jsonrpc: String,
    pub error: JsonRpcErrorDetail,
}

/// The `error` field inside a [`JsonRpcError`].
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JsonRpcErrorDetail {
    /// A number that indicates the error type.
    pub code: i32,
    /// A short description of the error.
    pub message: String,
    /// Additional information about the error.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

// ===========================================================================
// Common Primitives
// ===========================================================================

/// The sender or recipient of messages and data in a conversation.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Assistant,
    User,
}

/// An opaque token used to represent a cursor for pagination.
pub type Cursor = String;

/// Logging severity levels (RFC 5424 syslog).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LoggingLevel {
    Debug,
    Info,
    Notice,
    Warning,
    Error,
    Critical,
    Alert,
    Emergency,
}

/// Optional annotations for the client. Used to inform how objects are
/// displayed or prioritised.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Annotations {
    /// Describes who the intended audience of this data is.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub audience: Vec<Role>,

    /// Importance from 0.0 (least) to 1.0 (most).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<f64>,
}

// ===========================================================================
// Implementation / Server Info
// ===========================================================================

/// Describes the name and version of an MCP implementation (client or server).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Implementation {
    /// Programmatic name / identifier.
    pub name: String,

    /// Version string.
    pub version: String,

    /// Human-readable display title (2025-06-18+).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

// ===========================================================================
// Content Types
// ===========================================================================

/// Text content provided to or from an LLM.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TextContent {
    /// Content type discriminator — always `"text"`.
    #[serde(rename = "type")]
    pub type_: String,

    /// The text content of the message.
    pub text: String,

    /// Optional annotations for the client.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
}

impl TextContent {
    /// Create a new `TextContent` with the given text.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            type_: "text".to_string(),
            text: text.into(),
            annotations: None,
        }
    }
}

/// An image provided to or from an LLM.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ImageContent {
    #[serde(rename = "type")]
    pub type_: String,

    /// The base64-encoded image data.
    pub data: String,

    /// The MIME type of the image (e.g., `image/png`).
    #[serde(rename = "mimeType")]
    pub mime_type: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
}

/// Audio content provided to or from an LLM.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AudioContent {
    #[serde(rename = "type")]
    pub type_: String,

    /// The base64-encoded audio data.
    pub data: String,

    /// The MIME type of the audio.
    #[serde(rename = "mimeType")]
    pub mime_type: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
}

/// A link to a resource, used in content blocks.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ResourceLink {
    #[serde(rename = "type")]
    pub type_: String,

    /// The URI of the resource.
    pub uri: String,

    /// A human-readable name for the resource.
    pub name: String,

    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Human-readable display title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// MIME type of the resource.
    #[serde(rename = "mimeType", default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,

    /// Size in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,

    #[serde(rename = "_meta", default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

/// The contents of a resource embedded into a prompt or tool call result.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EmbeddedResource {
    #[serde(rename = "type")]
    pub type_: String,

    /// The resource contents (text or blob).
    pub resource: ResourceContents,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
}

/// The contents of a specific resource — either text or binary.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum ResourceContents {
    Text(TextResourceContents),
    Blob(BlobResourceContents),
}

/// Text contents of a resource.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TextResourceContents {
    /// The URI of this resource.
    pub uri: String,

    /// The text of the item.
    pub text: String,

    /// The MIME type of this resource, if known.
    #[serde(rename = "mimeType", default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// Binary (blob) contents of a resource.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BlobResourceContents {
    /// The URI of this resource.
    pub uri: String,

    /// A base64-encoded string representing the binary data.
    pub blob: String,

    /// The MIME type of this resource, if known.
    #[serde(rename = "mimeType", default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// A content block in an MCP message or tool result.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text(TextContent),

    #[serde(rename = "image")]
    Image(ImageContent),

    #[serde(rename = "audio")]
    Audio(AudioContent),

    #[serde(rename = "resource_link")]
    ResourceLink(ResourceLink),

    #[serde(rename = "resource")]
    EmbeddedResource(EmbeddedResource),
}

impl From<TextContent> for ContentBlock {
    fn from(value: TextContent) -> Self {
        Self::Text(value)
    }
}

// ===========================================================================
// Tools
// ===========================================================================

/// Definition for a tool the client can call.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Tool {
    /// The programmatic name of the tool.
    pub name: String,

    /// A human-readable description of the tool.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// JSON Schema object defining the expected parameters for the tool.
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,

    /// Optional JSON Schema describing the tool's structured output (2025-06-18+).
    #[serde(
        rename = "outputSchema",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub output_schema: Option<serde_json::Value>,

    /// Human-readable display title (2025-06-18+).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Optional behavioural annotations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: Option<ToolAnnotations>,

    /// Protocol metadata.
    #[serde(rename = "_meta", default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

/// Additional properties describing a Tool to clients.
///
/// NOTE: all properties in ToolAnnotations are **hints**. They are not
/// guaranteed to provide a faithful description of tool behavior.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ToolAnnotations {
    /// If true, the tool does not modify its environment.
    #[serde(
        rename = "readOnlyHint",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub read_only_hint: Option<bool>,

    /// If true, the tool may perform destructive updates.
    #[serde(
        rename = "destructiveHint",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub destructive_hint: Option<bool>,

    /// If true, calling the tool repeatedly with the same arguments has no
    /// additional effect.
    #[serde(
        rename = "idempotentHint",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub idempotent_hint: Option<bool>,

    /// If true, this tool may interact with an "open world" of external entities.
    #[serde(
        rename = "openWorldHint",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub open_world_hint: Option<bool>,

    /// A human-readable title for the tool.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

// ===========================================================================
// Tool Call (tools/call)
// ===========================================================================

/// Parameters for a `tools/call` request.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CallToolRequestParams {
    /// The name of the tool to call.
    pub name: String,

    /// Arguments to pass to the tool (JSON object).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arguments: Option<serde_json::Value>,

    /// Protocol metadata.
    #[serde(rename = "_meta", default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

/// The server's response to a `tools/call` request.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CallToolResult {
    /// Content blocks representing the tool result.
    pub content: Vec<ContentBlock>,

    /// Whether the tool call ended in an error.
    #[serde(
        rename = "isError",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub is_error: Option<bool>,

    /// Optional structured (JSON) result of the tool call (2025-06-18+).
    #[serde(
        rename = "structuredContent",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub structured_content: Option<serde_json::Value>,

    /// Protocol metadata.
    #[serde(rename = "_meta", default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

// ===========================================================================
// List Tools (tools/list)
// ===========================================================================

/// Result of a `tools/list` request.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ListToolsResult {
    /// The tools available on this server.
    pub tools: Vec<Tool>,

    /// Pagination cursor for the next page, if any.
    #[serde(
        rename = "nextCursor",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub next_cursor: Option<Cursor>,

    /// Protocol metadata.
    #[serde(rename = "_meta", default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

// ===========================================================================
// Resources
// ===========================================================================

/// A known resource that the server is capable of reading.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Resource {
    /// The URI of this resource.
    pub uri: String,

    /// A human-readable name for this resource.
    pub name: String,

    /// A description of what this resource represents.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The MIME type of this resource, if known.
    #[serde(rename = "mimeType", default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,

    /// The size of the raw resource content in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
}

/// A template description for resources available on the server.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ResourceTemplate {
    /// A URI template (RFC 6570) for constructing resource URIs.
    #[serde(rename = "uriTemplate")]
    pub uri_template: String,

    /// A human-readable name for the type of resource.
    pub name: String,

    /// A description of what this template is for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The MIME type for all resources that match this template.
    #[serde(rename = "mimeType", default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
}

/// Result of a `resources/list` request.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ListResourcesResult {
    pub resources: Vec<Resource>,

    #[serde(
        rename = "nextCursor",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub next_cursor: Option<Cursor>,

    #[serde(rename = "_meta", default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

/// Result of a `resources/templates/list` request.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ListResourceTemplatesResult {
    #[serde(rename = "resourceTemplates")]
    pub resource_templates: Vec<ResourceTemplate>,

    #[serde(
        rename = "nextCursor",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub next_cursor: Option<Cursor>,

    #[serde(rename = "_meta", default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

/// Result of a `resources/read` request.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReadResourceResult {
    pub contents: Vec<ResourceContents>,

    #[serde(rename = "_meta", default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

// ===========================================================================
// Prompts
// ===========================================================================

/// A prompt or prompt template that the server offers.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Prompt {
    /// The name of the prompt or prompt template.
    pub name: String,

    /// An optional description of what this prompt provides.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// A list of arguments to use for templating the prompt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<PromptArgument>>,
}

/// Describes an argument that a prompt can accept.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PromptArgument {
    /// The name of the argument.
    pub name: String,

    /// A human-readable description of the argument.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Whether this argument must be provided.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}

/// A message returned as part of a prompt.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PromptMessage {
    pub role: Role,
    pub content: ContentBlock,
}

/// Result of a `prompts/list` request.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ListPromptsResult {
    pub prompts: Vec<Prompt>,

    #[serde(
        rename = "nextCursor",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub next_cursor: Option<Cursor>,

    #[serde(rename = "_meta", default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

/// Result of a `prompts/get` request.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetPromptResult {
    /// An optional description for the prompt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The prompt messages.
    pub messages: Vec<PromptMessage>,

    #[serde(rename = "_meta", default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

// ===========================================================================
// Sampling
// ===========================================================================

/// Describes a message issued to or received from an LLM API.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SamplingMessage {
    pub role: Role,
    pub content: ContentBlock,
}

// ===========================================================================
// Completion
// ===========================================================================

/// The server's response to a `completion/complete` request.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CompleteResult {
    pub completion: CompleteResultCompletion,

    #[serde(rename = "_meta", default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

/// The completion values and metadata.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CompleteResultCompletion {
    /// The completion values.
    pub values: Vec<String>,

    /// The total number of completion options available (may exceed `values.len()`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,

    /// Indicates whether there are additional completion options beyond those provided.
    #[serde(rename = "hasMore", default, skip_serializing_if = "Option::is_none")]
    pub has_more: Option<bool>,
}

// ===========================================================================
// Roots
// ===========================================================================

/// Represents a root directory or file that the server can operate on.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Root {
    /// The URI identifying the root. Must start with `file://`.
    pub uri: String,

    /// An optional name for the root.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

// ===========================================================================
// Initialize
// ===========================================================================

/// Parameters sent by the client in an `initialize` request.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InitializeRequestParams {
    /// The MCP protocol version the client supports.
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,

    /// Client capabilities.
    pub capabilities: ClientCapabilities,

    /// Information about the connecting client.
    #[serde(rename = "clientInfo")]
    pub client_info: Implementation,

    /// Protocol metadata.
    #[serde(rename = "_meta", default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

/// Capabilities a client may support.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ClientCapabilities {
    /// Experimental, non-standard capabilities.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub experimental: Option<HashMap<String, serde_json::Value>>,

    /// Present if the client supports listing roots.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roots: Option<ClientCapabilitiesRoots>,

    /// Present if the client supports sampling from an LLM.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sampling: Option<serde_json::Value>,
}

/// Roots capability details.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ClientCapabilitiesRoots {
    /// Whether the client supports notifications for changes to the roots list.
    #[serde(
        rename = "listChanged",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub list_changed: Option<bool>,
}

/// The server's response to an `initialize` request.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InitializeResult {
    /// The MCP protocol version the server wants to use.
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,

    /// Server capabilities.
    pub capabilities: ServerCapabilities,

    /// Information about the server implementation.
    #[serde(rename = "serverInfo")]
    pub server_info: Implementation,

    /// Optional instructions / hints for the LLM.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,

    /// Protocol metadata.
    #[serde(rename = "_meta", default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

// ===========================================================================
// Server Capabilities
// ===========================================================================

/// Capabilities that a server may advertise.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ServerCapabilities {
    /// Present if the server supports argument autocompletion suggestions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completions: Option<serde_json::Value>,

    /// Experimental, non-standard capabilities.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub experimental: Option<HashMap<String, serde_json::Value>>,

    /// Present if the server supports sending log messages to the client.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logging: Option<serde_json::Value>,

    /// Present if the server offers any prompt templates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompts: Option<ServerCapabilitiesPrompts>,

    /// Present if the server offers any resources to read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resources: Option<ServerCapabilitiesResources>,

    /// Present if the server offers any tools to call.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<ServerCapabilitiesTools>,
}

/// Prompt-related capability flags.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ServerCapabilitiesPrompts {
    /// Whether the server supports notifications for changes to the prompt list.
    #[serde(
        rename = "listChanged",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub list_changed: Option<bool>,
}

/// Resource-related capability flags.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ServerCapabilitiesResources {
    /// Whether the server supports notifications for changes to the resource list.
    #[serde(
        rename = "listChanged",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub list_changed: Option<bool>,

    /// Whether the server supports subscribing to resource updates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subscribe: Option<bool>,
}

/// Tool-related capability flags.
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

// ===========================================================================
// ServerResult — typed enum of all possible server responses
// ===========================================================================

/// All possible result types a server can return.
///
/// This replaces the previous `subtype_N` flattened struct approach with a
/// proper tagged enum, making dispatch explicit and type-safe.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum ServerResult {
    Initialize(InitializeResult),
    ListTools(ListToolsResult),
    CallTool(CallToolResult),
    ListResources(ListResourcesResult),
    ListResourceTemplates(ListResourceTemplatesResult),
    ReadResource(ReadResourceResult),
    ListPrompts(ListPromptsResult),
    GetPrompt(GetPromptResult),
    Complete(CompleteResult),
    Empty(EmptyResult),
}

/// An empty result (e.g., for `ping` responses).
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct EmptyResult {
    #[serde(rename = "_meta", default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}
