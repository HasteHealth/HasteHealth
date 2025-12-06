use crate::mcp::schemas::schema_2025_11_25::{
    CallToolRequestParams, InitializeRequestParams, RequestId,
};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct InitializeRequest {
    pub id: Option<RequestId>,
    pub jsonrpc: ::std::string::String,
    pub params: InitializeRequestParams,
}
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct PingRequest {}
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ListToolsRequest {
    pub id: Option<RequestId>,
}
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct InitializeNotification {
    pub id: Option<RequestId>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct CallToolRequest {
    pub id: Option<RequestId>,
    pub jsonrpc: ::std::string::String,
    pub params: CallToolRequestParams,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(tag = "method")]
pub enum MCPRequest {
    #[serde(rename = "ping")]
    Ping(PingRequest),
    #[serde(rename = "initialize")]
    Initialize(InitializeRequest),
    #[serde(rename = "notifications/initialized")]
    InitializedNotification(InitializeNotification),

    #[serde(rename = "tools/list")]
    ListTools(ListToolsRequest),
    #[serde(rename = "tools/call")]
    ToolsCall(CallToolRequest),
}
