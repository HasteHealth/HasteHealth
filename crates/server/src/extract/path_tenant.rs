use axum::{
    extract::{FromRequestParts, Path},
    http::request::Parts,
    response::{IntoResponse, Response},
};
use oxidized_repository::{ProjectId, TenantId};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct PathTenant {
    pub tenant: TenantId,
    pub project: ProjectId,
}

impl<S: Send + Sync> FromRequestParts<S> for PathTenant {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(tenant_information) = Path::<PathTenant>::from_request_parts(parts, state)
            .await
            .map_err(|err| err.into_response())?;

        Ok(tenant_information)
    }
}
