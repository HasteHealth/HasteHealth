use std::sync::Arc;

use crate::auth_n::certificates::{JSONWebKeySet, get_certification_provider};
use axum::Json;
use axum_extra::routing::TypedPath;

#[derive(TypedPath)]
#[typed_path("/certs/jwks")]
pub struct JWKSPath;

pub async fn jwks_get(_: JWKSPath) -> Json<Arc<JSONWebKeySet>> {
    let keyset = get_certification_provider().jwk_set();
    Json(keyset)
}
