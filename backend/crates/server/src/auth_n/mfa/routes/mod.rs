use crate::services::ServerState;
use axum::Router;
use axum_extra::routing::RouterExt;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::Repository;
use std::sync::Arc;

mod activate;
mod create;

pub fn create_router<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _state: Arc<ServerState<Repo, Search, Terminology>>,
) -> Router<Arc<ServerState<Repo, Search, Terminology>>> {
    Router::new()
        .typed_post(create::create_post)
        .typed_get(activate::activate_get)
        .typed_post(activate::activate_post)
}
