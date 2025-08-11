use std::sync::Arc;

use crate::{AppState, auth_n::oidc::middleware::OIDCParameters, server_client::ServerCTX};
use axum::{
    Extension, RequestPartsExt,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use oxidized_fhir_client::FHIRClient;
use oxidized_fhir_model::r4::types::{ClientApplication, Resource, ResourceType};
use oxidized_fhir_repository::{Author, FHIRRepository, ProjectId, TenantId};
use oxidized_fhir_search::SearchEngine;

pub struct OIDCClientApplication(pub ClientApplication);

impl<Repo, Search> FromRequestParts<Arc<AppState<Repo, Search>>> for OIDCClientApplication
where
    Repo: FHIRRepository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
{
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState<Repo, Search>>,
    ) -> Result<Self, Self::Rejection> {
        let Extension(oidc_params) = parts
            .extract::<Extension<OIDCParameters>>()
            .await
            .map_err(|err| err.into_response())?;

        let ctx = ServerCTX {
            tenant: TenantId::new("tenant".to_string()),
            project: ProjectId::new("project".to_string()),
            fhir_version: oxidized_fhir_repository::SupportedFHIRVersions::R4,
            author: Author {
                id: "anonymous".to_string(),
                kind: "Membership".to_string(),
            },
        };

        let client_app = state
            .fhir_client
            .read(
                ctx,
                ResourceType::new("ClientApplication".to_string()).unwrap(),
                oidc_params
                    .parameters
                    .get("client_id")
                    .cloned()
                    .unwrap_or_default(),
            )
            .await
            .map_err(|err| err.into_response())?;

        if let Some(Resource::ClientApplication(client_app)) = client_app {
            Ok(OIDCClientApplication(client_app))
        } else {
            Err((StatusCode::NOT_FOUND, "Client application not found").into_response())
        }
    }
}
