use std::sync::Arc;

use crate::{
    auth_n::certificates::{JSONWebKeySet, get_certification_provider},
    services::ServerState,
};
use axum::{Json, extract::State};
use axum_extra::routing::TypedPath;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::Repository;

#[derive(TypedPath)]
#[typed_path("/certs/jwks")]
pub struct JWKSPath;

pub async fn jwks_get<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    _: JWKSPath,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
) -> Json<Arc<JSONWebKeySet>> {
    let keyset = get_certification_provider(state.config.as_ref()).jwk_set();
    Json(keyset)
}
