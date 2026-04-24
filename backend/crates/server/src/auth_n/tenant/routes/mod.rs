use crate::services::AppState;
use axum::Router;
use axum_extra::routing::RouterExt as _;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::Repository;
use std::sync::Arc;

mod project_select;

pub fn create_router<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>() -> Router<Arc<AppState<Repo, Search, Terminology>>> {
    Router::new().nest(
        "/interactions",
        Router::new().typed_get(project_select::project_get),
    )
}
