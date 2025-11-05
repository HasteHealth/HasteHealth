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
use oxidized_fhir_model::r4::generated::resources::{Resource, ResourceType};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_repository::Repository;
use serde::Deserialize;

#[derive(TypedPath, Deserialize)]
#[typed_path("/{identity_provider_id}")]
pub struct FederatedInitiate {
    pub identity_provider_id: String,
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
    OIDCClientApplication(client_app): OIDCClientApplication,
    uri: OriginalUri,
) -> Result<Markup, OperationOutcomeError> {
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

    todo!();
}
