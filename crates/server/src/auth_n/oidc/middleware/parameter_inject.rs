use std::collections::HashMap;

use axum::{
    body::{Body, to_bytes},
    extract::Request,
    response::Response,
};
use oxidized_fhir_operation_error::OperationOutcomeError;
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct OIDCParameters(pub HashMap<String, String>);

pub async fn parameter_inject_middleware(
    request: Request<Body>,
    next: axum::middleware::Next,
) -> Result<Response<Body>, OperationOutcomeError> {
    let (parts, body) = request.into_parts();
    let bytes = to_bytes(body, 10000).await.unwrap();

    let oidc_params = serde_json::from_slice::<OIDCParameters>(&bytes).unwrap();

    let new_body = Body::from(bytes);

    let mut request2 = Request::from_parts(parts, new_body);
    request2.extensions_mut().insert(oidc_params);

    Ok(next.run(request2).await)
}
