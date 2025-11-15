use crate::auth_n::certificates::{JSONWebKeySet, JWK_SET};
use axum::Json;
use axum_extra::routing::TypedPath;
use haste_fhir_operation_error::OperationOutcomeError;

#[derive(TypedPath)]
#[typed_path("/certs/jwks")]
pub struct JWKSPath;

pub async fn jwks_get(_: JWKSPath) -> Result<Json<&'static JSONWebKeySet>, OperationOutcomeError> {
    Ok(Json(&*JWK_SET))
}
