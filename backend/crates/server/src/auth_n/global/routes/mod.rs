use crate::services::AppState;
use axum::Router;
use axum_extra::routing::RouterExt;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::Repository;
use std::sync::Arc;

mod signup;
mod tenant_select;

pub fn create_router<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _state: Arc<AppState<Repo, Search, Terminology>>,
) -> Router<Arc<AppState<Repo, Search, Terminology>>> {
    Router::new()
        .typed_get(tenant_select::tenant_select_get)
        .typed_post(tenant_select::tenant_select_post)
}
