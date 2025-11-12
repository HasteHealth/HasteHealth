use crate::services::AppState;
use axum::Router;
use axum_extra::routing::RouterExt;
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_repository::Repository;
use std::sync::Arc;

mod callback;
mod initiate;

pub fn federated_router<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>() -> Router<Arc<AppState<Repo, Search, Terminology>>> {
    let router = Router::new().typed_get(initiate::federated_initiate);
    router
}
