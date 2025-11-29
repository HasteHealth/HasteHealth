use crate::auth_n::certificates::{JSONWebKeySet, JWK_SET};
use axum::Json;
use axum_extra::routing::TypedPath;

#[derive(TypedPath)]
#[typed_path("/certs/jwks")]
pub struct JWKSPath;

pub async fn jwks_get(_: JWKSPath) -> Json<&'static JSONWebKeySet> {
    Json(&*JWK_SET)
}
