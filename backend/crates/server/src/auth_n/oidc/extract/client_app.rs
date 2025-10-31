use std::sync::Arc;

use crate::{
    auth_n::oidc::{hardcoded_clients::get_hardcoded_clients, middleware::OIDCParameters},
    extract::path_tenant::{ProjectIdentifier, TenantIdentifier},
    fhir_client::ServerCTX,
    services::AppState,
};
use axum::{
    Extension, RequestPartsExt,
    extract::FromRequestParts,
    http::request::Parts,
    response::{IntoResponse, Response},
};
use axum_extra::extract::Cached;
use oxidized_fhir_client::FHIRClient;
use oxidized_fhir_model::r4::generated::{
    resources::{ClientApplication, Resource, ResourceType},
    terminology::IssueType,
};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_jwt::{ProjectId, TenantId};
use oxidized_repository::Repository;

pub async fn find_client_app<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    state: &Arc<AppState<Repo, Search, Terminology>>,
    tenant: TenantId,
    project: ProjectId,
    client_id: String,
) -> Result<ClientApplication, OperationOutcomeError> {
    let hardcoded_clients = get_hardcoded_clients(&*state.config);

    if let Some(client) = hardcoded_clients
        .into_iter()
        .find(|client| client.id.as_ref() == Some(&client_id))
    {
        Ok(client)
    } else {
        let client_app = state
            .fhir_client
            .read(
                Arc::new(ServerCTX::system(tenant, project)),
                ResourceType::ClientApplication,
                client_id,
            )
            .await?;

        if let Some(Resource::ClientApplication(client_app)) = client_app {
            Ok(client_app)
        } else {
            Err(OperationOutcomeError::error(
                IssueType::NotFound(None),
                "Client application not found".to_string(),
            ))
        }
    }
}

#[allow(unused)]
pub struct OIDCClientApplication(pub ClientApplication);

impl<Repo, Search, Terminology> FromRequestParts<Arc<AppState<Repo, Search, Terminology>>>
    for OIDCClientApplication
where
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState<Repo, Search, Terminology>>,
    ) -> Result<Self, Self::Rejection> {
        let Extension(oidc_params) = parts
            .extract::<Extension<OIDCParameters>>()
            .await
            .map_err(|err| err.into_response())?;

        let Cached(TenantIdentifier { tenant }) =
            Cached::<TenantIdentifier>::from_request_parts(parts, state)
                .await
                .map_err(|err| err.into_response())?;

        let Cached(ProjectIdentifier { project }) =
            Cached::<ProjectIdentifier>::from_request_parts(parts, state)
                .await
                .map_err(|err| err.into_response())?;

        let client_app = find_client_app(
            state,
            tenant,
            project,
            oidc_params
                .parameters
                .get("client_id")
                .cloned()
                .unwrap_or_default(),
        )
        .await
        .map_err(|err| err.into_response())?;

        Ok(OIDCClientApplication(client_app))
    }
}
