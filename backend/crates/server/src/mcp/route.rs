use crate::{
    extract::path_tenant::{ProjectIdentifier, TenantIdentifier},
    fhir_client::ServerCTX,
    mcp::{
        error::MCPError,
        operations,
        schemas::schema_2025_11_25::{
            ClientNotification, ClientRequest, InitializeRequestParams, RequestId, ServerResult,
        },
    },
    services::AppState,
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
use haste_jwt::claims::UserTokenClaims;
use haste_repository::{Repository, types::SupportedFHIRVersions};
use std::sync::Arc;

#[derive(serde::Serialize, Debug)]
pub struct JSONRPCResult<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<RequestId>,
    jsonrpc: String,
    result: T,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(tag = "method")]
pub enum MCPRequest {
    #[serde(rename = "ping")]
    Ping {},
    #[serde(rename = "initialize")]
    Initialize {
        id: Option<RequestId>,
        jsonrpc: ::std::string::String,
        params: InitializeRequestParams,
    },
    #[serde(rename = "tools/list")]
    ListTools { id: Option<RequestId> },
    #[serde(rename = "notifications/initialized")]
    InitializedNotification { id: Option<RequestId> },
}

pub async fn mcp_handler<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    Cached(ProjectIdentifier { project }): Cached<ProjectIdentifier>,
    State(state): State<Arc<AppState<Repo, Search, Terminology>>>,
    Extension(claims): Extension<Arc<UserTokenClaims>>,
    Json(mcp_request): Json<MCPRequest>,
) -> Result<Response, MCPError<serde_json::Value>> {
    let ctx = Arc::new(ServerCTX::new(
        tenant,
        project,
        SupportedFHIRVersions::R4,
        claims.clone(),
        state.fhir_client.clone(),
    ));

    match mcp_request {
        MCPRequest::Initialize {
            id,
            jsonrpc,
            params,
        } => {
            let result = ServerResult {
                subtype_1: Some(operations::initialize(ctx).await?),
                ..ServerResult::default()
            };
            Ok(Json(JSONRPCResult {
                id: id,
                result,
                jsonrpc: "2.0".to_string(),
            })
            .into_response())
        }
        MCPRequest::ListTools { id } => Ok(Json(JSONRPCResult {
            id: id.clone(),
            result: ServerResult {
                subtype_7: Some(operations::list_tools(ctx).await?),
                ..ServerResult::default()
            },
            jsonrpc: "2.0".to_string(),
        })
        .into_response()),
        MCPRequest::InitializedNotification { id } => Ok(StatusCode::OK.into_response()),
        _ => Err(OperationOutcomeError::error(
            IssueType::NotSupported(None),
            "Request not implemented".to_string(),
        )
        .into()),
    }
}
