use crate::{
    auth_n::oidc::{
        middleware::OIDCParameterInjectLayer,
        routes::{AUTHORIZE_PARAMETERS, LOGOUT_PARAMETERS},
    },
    services::AppState,
};
use axum::Router;
use axum_extra::routing::RouterExt;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::Repository;
use std::sync::Arc;
use tower::ServiceBuilder;

mod login;
mod logout;
mod password_reset;

pub fn interactions_router<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>() -> Router<Arc<AppState<Repo, Search, Terminology>>> {
    let password_reset_routes = Router::new()
        .typed_get(password_reset::password_reset_initiate_get)
        .typed_post(password_reset::password_reset_initiate_post)
        .typed_get(password_reset::password_reset_verify_get)
        .typed_post(password_reset::password_reset_verify_post);

    let login_routes = Router::new()
        .typed_get(login::login_get)
        .typed_post(login::login_post)
        .route_layer(ServiceBuilder::new().layer(OIDCParameterInjectLayer::new(
            (*AUTHORIZE_PARAMETERS).clone(),
        )));

    let logout_routes = Router::new()
        .typed_post(logout::logout)
        .typed_get(logout::logout)
        .route_layer(
            ServiceBuilder::new()
                .layer(OIDCParameterInjectLayer::new((*LOGOUT_PARAMETERS).clone())),
        );

    Router::new()
        .merge(login_routes)
        .merge(logout_routes)
        .merge(password_reset_routes)
}
