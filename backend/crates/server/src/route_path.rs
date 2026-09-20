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

/// Appends path segments to a URL: `…/api/v1/fhir` and `["Patient", "1"]` give
/// `…/api/v1/fhir/Patient/1`.
///
/// Not `Url::join`, which resolves its argument relative to the URL's last path
/// segment and would drop `fhir`. Also tolerates a trailing slash on the base,
/// and escapes the segments.
pub fn append_path_segments<'a>(
    url: &Url,
    segments: impl IntoIterator<Item = &'a str>,
) -> Option<Url> {
    let mut url = url.clone();

    url.path_segments_mut()
        .ok()?
        .pop_if_empty()
        .extend(segments);

    Some(url)
}

pub fn api_fhir_root_url(
    api_url_string: &str,
    tenant: &TenantId,
    project: &ProjectId,
) -> Result<Url, OperationOutcomeError> {
    let api_url = Url::parse(api_url_string).map_err(|e| {
        tracing::error!("Failed to parse API URL: {:?}", e);
        OperationOutcomeError::error(
            IssueType::invalid(),
            "Invalid API URL configured".to_string(),
        )
    })?;

    let fhir_url = api_url
        .join(api_v1_fhir_path(tenant, project).to_str().unwrap())
        .map_err(|e| {
            tracing::error!("Failed to derive FHIR URL: {:?}", e);
            OperationOutcomeError::error(
                IssueType::invalid(),
                "Invalid API URL configured".to_string(),
            )
        })?;

    Ok(fhir_url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segments_are_appended_under_the_last_path_segment() {
        let root = Url::parse("https://api.haste.health/w/acme/default/api/v1/fhir").unwrap();

        assert_eq!(
            append_path_segments(&root, ["Patient", "123"])
                .as_ref()
                .map(Url::as_str),
            Some("https://api.haste.health/w/acme/default/api/v1/fhir/Patient/123")
        );
    }

    /// A base given with a trailing slash addresses the same thing, without
    /// doubling the separator.
    #[test]
    fn a_trailing_slash_on_the_base_is_ignored() {
        let with = Url::parse("https://api.haste.health/w/acme/default/api/v1/fhir/").unwrap();
        let without = Url::parse("https://api.haste.health/w/acme/default/api/v1/fhir").unwrap();

        assert_eq!(
            append_path_segments(&with, ["metadata"]),
            append_path_segments(&without, ["metadata"])
        );
    }

    /// The FHIR endpoint carries no version: it is a property of the project.
    /// This string is also the token `aud`, so a change here invalidates every
    /// token already issued.
    #[test]
    fn fhir_root_url_has_no_version_segment() {
        let url = api_fhir_root_url(
            "https://api.haste.health",
            &TenantId::new("acme".to_string()),
            &ProjectId::new("default".to_string()),
        )
        .unwrap();

        assert_eq!(
            url.as_str(),
            "https://api.haste.health/w/acme/default/api/v1/fhir"
        );
    }
}
