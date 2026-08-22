use crate::{services::ServerState, static_assets::StaticFile, tenants::read_tenant};
use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderValue, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::TenantId;
use haste_repository::Repository;
use std::sync::Arc;

pub async fn logo<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    Path(tenant_id): Path<TenantId>,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
) -> Result<Response, OperationOutcomeError> {
    let tenant = read_tenant(&state, &tenant_id).await?;
    let Some(data) = tenant.logo_data.filter(|data| !data.is_empty()) else {
        return Ok(StaticFile("img/logo.svg".to_string()).into_response());
    };

    let content_type = tenant
        .logo_content_type
        .as_deref()
        .filter(|content_type| content_type.starts_with("image/"))
        .ok_or_else(|| {
            OperationOutcomeError::error(
                haste_fhir_model::r4::generated::terminology::IssueType::invalid(),
                "Tenant logo has an invalid image content type.".to_string(),
            )
        })?;

    let mut response = Response::new(Body::from(data));
    response.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_str(content_type).map_err(|_| {
            OperationOutcomeError::error(
                haste_fhir_model::r4::generated::terminology::IssueType::invalid(),
                "Tenant logo has an invalid content type header.".to_string(),
            )
        })?,
    );
    Ok(response)
}
