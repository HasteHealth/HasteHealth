use crate::{
    auth_n::oidc::{middleware::OIDCParameterInjectLayer, routes::AUTHORIZE_PARAMETERS},
    services::AppState,
};
use axum::Router;
use axum_extra::routing::RouterExt;
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_repository::Repository;
use std::sync::Arc;
use tower::ServiceBuilder;

mod callback;
mod initiate;

pub use initiate::FederatedInitiate;

pub fn federated_router<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>() -> Router<Arc<AppState<Repo, Search, Terminology>>> {
    let router = Router::new().typed_get(callback::federated_callback).merge(
        // Only initiate route needs authorize params (we redirect back to authorize in callback with redirect uri stored in session).
        Router::new()
            .typed_get(initiate::federated_initiate)
            .route_layer(ServiceBuilder::new().layer(OIDCParameterInjectLayer::new(
                (*AUTHORIZE_PARAMETERS).clone(),
            ))),
    );

    router
}
