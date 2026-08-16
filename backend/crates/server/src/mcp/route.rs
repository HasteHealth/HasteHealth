use crate::{
    auth_n::middleware::jwt::User,
    extract::path_tenant::{ProjectIdentifier, TenantIdentifier},
    fhir_client::ServerCTX,
    mcp::{
        error::MCPError,
        operations,
        request::MCPRequest,
        schemas::types::{RequestId, ServerResult},
    },
    services::ServerState,
};
use axum::{
    Extension, Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_extra::extract::Cached;
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::{Repository, types::SupportedFHIRVersions, utilities::generate_id};
use std::sync::Arc;

#[derive(serde::Serialize, Debug)]
pub struct JSONRPCResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<RequestId>,
    jsonrpc: String,
    result: ServerResult,
}

pub async fn mcp_handler<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    Cached(ProjectIdentifier { project }): Cached<ProjectIdentifier>,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
    Extension(user): Extension<Arc<User>>,
    Json(mcp_request): Json<MCPRequest>,
) -> Result<Response, MCPError<serde_json::Value>> {
    let ctx = Arc::new(
        ServerCTX::new(
            tenant,
            project,
            SupportedFHIRVersions::R4,
            user.clone(),
            state.fhir_client.clone(),
            state.rate_limit.clone(),
        )
        .with_tracing_id(Some(format!("mcp-{}", generate_id(Some(8))))),
    );

    let api_uri = &state.config.api_uri;

    match mcp_request {
        MCPRequest::Initialize(initialize_request) => {
            let result = operations::initialize(ctx, &initialize_request).await?;
            Ok(Json(JSONRPCResult {
                id: initialize_request.id.clone(),
                result: ServerResult::Initialize(result),
                jsonrpc: "2.0".to_string(),
            })
            .into_response())
        }
        MCPRequest::ListTools(list_tools_request) => {
            let result = operations::list_tools(ctx, &list_tools_request, api_uri).await?;
            Ok(Json(JSONRPCResult {
                id: list_tools_request.id.clone(),
                result: ServerResult::ListTools(result),
                jsonrpc: "2.0".to_string(),
            })
            .into_response())
        }
        MCPRequest::InitializedNotification(_initialized_notification) => {
            Ok(StatusCode::OK.into_response())
        }
        MCPRequest::ToolsCall(tools_call_request) => {
            let id = tools_call_request.id.clone();
            let result = operations::tools_call(ctx, tools_call_request, api_uri).await?;

            Ok(Json(JSONRPCResult {
                id,
                result: ServerResult::CallTool(result),
                jsonrpc: "2.0".to_string(),
            })
            .into_response())
        }
        _ => Err(OperationOutcomeError::error(
            IssueType::not_supported(),
            "Request not implemented".to_string(),
        )
        .into()),
    }
}
