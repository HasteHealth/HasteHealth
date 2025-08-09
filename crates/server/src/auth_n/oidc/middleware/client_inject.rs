use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Request, State},
    http::Response,
};
use oxidized_fhir_client::FHIRClient;
use oxidized_fhir_model::r4::types::ResourceType;
use oxidized_fhir_repository::{Author, FHIRRepository, ProjectId, TenantId};
use oxidized_fhir_search::SearchEngine;

use crate::{AppState, server_client::ServerCTX};

pub async fn client_inject_middleware<
    Repo: FHIRRepository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
>(
    State(state): State<Arc<AppState<Repo, Search>>>,
    req: Request<Body>,
    next: axum::middleware::Next,
) -> Response<Body> {
    let ctx = ServerCTX {
        tenant: TenantId::new("tenant".to_string()),
        project: ProjectId::new("project".to_string()),
        fhir_version: oxidized_fhir_repository::SupportedFHIRVersions::R4,
        author: Author {
            id: "anonymous".to_string(),
            kind: "Membership".to_string(),
        },
    };

    let res = state
        .fhir_client
        .search_type(
            ctx,
            ResourceType::new("ClientApplication".to_string()).unwrap(),
            vec![],
        )
        .await;

    println!("{:?}", res);

    let response = next.run(req).await;

    // do something with `response`...

    response
}
