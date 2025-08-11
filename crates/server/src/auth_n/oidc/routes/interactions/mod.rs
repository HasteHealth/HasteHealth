use crate::AppState;
use axum::Router;
use axum_extra::routing::RouterExt;
use oxidized_fhir_repository::FHIRRepository;
use oxidized_fhir_search::SearchEngine;
use std::sync::Arc;

mod login;

pub fn interactions_router<
    Repo: FHIRRepository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
>() -> Router<Arc<AppState<Repo, Search>>> {
    Router::new()
        .typed_get(login::login_get)
        .typed_post(login::login_post)
}
