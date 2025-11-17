use crate::services::AppState;
use axum::Router;
use axum_extra::routing::RouterExt;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::Repository;
use std::sync::Arc;

pub mod routes;

pub fn create_router<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>() -> Router<Arc<AppState<Repo, Search, Terminology>>> {
    Router::new()
        .typed_get(routes::login::login_get)
        .typed_post(routes::login::login_post)
        .typed_post(routes::signup::signup_get)
}
