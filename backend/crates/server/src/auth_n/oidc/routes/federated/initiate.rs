use std::sync::Arc;

use crate::{
    auth_n::oidc::extract::client_app::OIDCClientApplication,
    extract::path_tenant::{Project, ProjectIdentifier, TenantIdentifier},
    fhir_client::ServerCTX,
    services::AppState,
};
use axum::extract::{OriginalUri, State};
use axum_extra::{extract::Cached, routing::TypedPath};
use maud::Markup;
use oxidized_fhir_client::FHIRClient;
use oxidized_fhir_model::r4::generated::{
    resources::{Project as FHIRProject, Resource, ResourceType},
    terminology::IssueType,
};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_repository::Repository;
use serde::Deserialize;
use url::Url;

#[derive(TypedPath, Deserialize)]
#[typed_path("/{identity_provider_id}")]
pub struct FederatedInitiate {
    pub identity_provider_id: String,
}

fn validate_identity_provider_in_project(
    identity_provider_id: &str,
    project: &FHIRProject,
) -> Result<(), OperationOutcomeError> {
    if let Some(identity_providers) = &project.identityProvider {
        for ip_ref in identity_providers {
            if let Some(ref_id) = &ip_ref.reference.as_ref().and_then(|r| r.value.as_ref()) {
                if ref_id.as_str() == &format!("IdentityProvider/{}", identity_provider_id) {
                    return Ok(());
                }
            }
        }
    }
    Err(OperationOutcomeError::error(
        IssueType::Forbidden(None),
        "The specified identity provider is not associated with the project.".to_string(),
    ))
}

pub async fn federated_initiate<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    FederatedInitiate {
        identity_provider_id,
    }: FederatedInitiate,
    State(state): State<Arc<AppState<Repo, Search, Terminology>>>,
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    Cached(ProjectIdentifier { project }): Cached<ProjectIdentifier>,
    Cached(Project(project_resource)): Cached<Project>,
    OIDCClientApplication(_client_app): OIDCClientApplication,
    _uri: OriginalUri,
) -> Result<Markup, OperationOutcomeError> {
    validate_identity_provider_in_project(&identity_provider_id, &project_resource)?;

    let identity_provider = state
        .fhir_client
        .read(
            Arc::new(ServerCTX::system(
                tenant,
                project,
                state.fhir_client.clone(),
            )),
            ResourceType::IdentityProvider,
            identity_provider_id,
        )
        .await?
        .and_then(|r| match r {
            Resource::IdentityProvider(ip) => Some(ip),
            _ => None,
        });

    if let Some(identity_provider) = identity_provider
        && let Some(oidc) = &identity_provider.oidc
    {
        let mut authorization_url = oidc
            .authorization_endpoint
            .value
            .as_ref()
            .and_then(|s| Url::parse(s).ok())
            .ok_or_else(|| {
                OperationOutcomeError::error(
                    IssueType::Invalid(None),
                    "Invalid authorization endpoint URL for identity provider".to_string(),
                )
            })?;
        let client_id = oidc.client.clientId.value.as_ref().ok_or_else(|| {
            OperationOutcomeError::error(
                IssueType::Invalid(None),
                "Missing client ID for identity provider.".to_string(),
            )
        })?;

        let scopes = oidc.scopes.as_ref().map(|s| {
            s.iter()
                .filter_map(|v| v.value.as_ref())
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(" ")
        });

        authorization_url.set_query(Some("response_type=code"));
        authorization_url
            .query_pairs_mut()
            .append_pair("client_id", client_id)
            .append_pair("scope", &scopes.unwrap_or_default());

        todo!();
    } else {
        return Err(OperationOutcomeError::error(
            IssueType::NotFound(None),
            "The specified identity provider was not found.".to_string(),
        ));
    }
}
