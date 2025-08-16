use crate::AppState;
use axum::Router;
use axum_extra::routing::RouterExt;
use oxidized_fhir_search::SearchEngine;
use oxidized_repository::Repository;
use std::sync::Arc;

mod login;
mod logout;

pub fn interactions_router<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
>() -> Router<Arc<AppState<Repo, Search>>> {
    Router::new()
        .typed_get(login::login_get)
        .typed_post(login::login_post)
        .typed_post(logout::logout)
        .typed_get(logout::logout)
}
