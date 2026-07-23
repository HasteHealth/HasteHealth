use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
// This module provides functions to generate routes for different entities in the system.
use haste_jwt::{ProjectId, TenantId};
use std::path::PathBuf;
use url::Url;

pub fn tenant_path(tenant: &TenantId) -> PathBuf {
    let mut path = PathBuf::new();
    path.push(format!("/w/{}", tenant));

    path
}

pub fn project_path(tenant: &TenantId, project: &ProjectId) -> PathBuf {
    let mut tenant_path = tenant_path(tenant);
    tenant_path.push(format!("{}", project));

    tenant_path
}

pub fn api_v1_path(tenant: &TenantId, project: &ProjectId) -> PathBuf {
    let mut project_path = project_path(tenant, project);
    project_path.push("api/v1");

    project_path
}

pub fn api_v1_fhir_path(tenant: &TenantId, project: &ProjectId) -> PathBuf {
    let mut api_v1_path = api_v1_path(tenant, project);
    api_v1_path.push("fhir");

    api_v1_path
}

pub fn api_v1_mcp_path(tenant: &TenantId, project: &ProjectId) -> PathBuf {
    let mut api_v1_path = api_v1_path(tenant, project);
    api_v1_path.push("mcp");

    api_v1_path
}

pub fn api_v1_oidc_path(tenant: &TenantId, project: &ProjectId) -> PathBuf {
    let mut api_v1_path = api_v1_path(tenant, project);
    api_v1_path.push("oidc");

    api_v1_path
}

pub fn api_v1_oidc_auth_path(tenant: &TenantId, project: &ProjectId) -> PathBuf {
    let mut api_v1_oidc_path = api_v1_oidc_path(tenant, project);
    api_v1_oidc_path.push("auth");

    api_v1_oidc_path
}

pub fn api_fhir_root_url(
    api_url_string: &str,
    tenant: &TenantId,
    project: &ProjectId,
) -> Result<Url, OperationOutcomeError> {
    let api_url = Url::parse(&api_url_string).map_err(|e| {
        tracing::error!("Failed to parse API URL: {:?}", e);
        OperationOutcomeError::error(
            IssueType::invalid(),
            "Invalid API URL configured".to_string(),
        )
    })?;

    let fhir_url = api_url
        .join(
            api_v1_fhir_path(tenant, project)
                .join("r4")
                .to_str()
                .unwrap(),
        )
        .map_err(|e| {
            tracing::error!("Failed to derive FHIR URL: {:?}", e);
            OperationOutcomeError::error(
                IssueType::invalid(),
                "Invalid API URL configured".to_string(),
            )
        })?;

    Ok(fhir_url)
}
