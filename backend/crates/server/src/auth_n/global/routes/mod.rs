use crate::services::ServerState;
use axum::Router;
use axum_extra::routing::RouterExt;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::Repository;
use std::sync::Arc;

mod flow;
mod login;
mod signup;
mod tenant_select;
mod verify;

pub fn create_router<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _state: Arc<ServerState<Repo, Search, Terminology>>,
) -> Router<Arc<ServerState<Repo, Search, Terminology>>> {
    Router::new()
        .typed_get(tenant_select::tenant_select_get)
        .typed_post(tenant_select::tenant_select_post)
        .typed_get(signup::global_signup_get)
        .typed_post(signup::global_signup_post)
        .typed_get(login::global_login_get)
        .typed_post(login::global_login_post)
        .typed_get(verify::verify_get)
        .typed_post(verify::verify_post)
        .typed_post(verify::resend_post)
        .typed_get(verify::tenant_get)
        .typed_post(verify::create_post)
}
