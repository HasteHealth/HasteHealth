use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Request, State},
    http::Response,
};
use oxidized_fhir_client::FHIRClient;
use oxidized_fhir_model::r4::types::{ClientApplication, Resource, ResourceType};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_repository::{Author, FHIRRepository, ProjectId, TenantId};
use oxidized_fhir_search::SearchEngine;

use crate::{AppState, server_client::ServerCTX};

pub async fn client_inject_middleware<
    Repo: FHIRRepository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
>(
    State(state): State<Arc<AppState<Repo, Search>>>,

    mut request: Request<Body>,
    next: axum::middleware::Next,
) -> Result<Response<Body>, OperationOutcomeError> {
    let ctx = ServerCTX {
        tenant: TenantId::new("tenant".to_string()),
        project: ProjectId::new("project".to_string()),
        fhir_version: oxidized_fhir_repository::SupportedFHIRVersions::R4,
        author: Author {
            id: "anonymous".to_string(),
            kind: "Membership".to_string(),
        },
    };

    let client_apps = state
        .fhir_client
        .search_type(
            ctx,
            ResourceType::new("ClientApplication".to_string()).unwrap(),
            vec![],
        )
        .await?
        .into_iter()
        .filter_map(|client_app| match client_app {
            Resource::ClientApplication(client_app) => Some(client_app),
            _ => None,
        })
        .collect();

    request
        .extensions_mut()
        .insert::<Vec<ClientApplication>>(client_apps);

    let response = next.run(request).await;

    Ok(response)
}
