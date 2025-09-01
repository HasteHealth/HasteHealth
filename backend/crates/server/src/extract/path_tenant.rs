use axum::{
    extract::{FromRequestParts, Path},
    http::request::Parts,
    response::{IntoResponse, Response},
};
use oxidized_repository::types::{ProjectId, TenantId};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Project {
    pub project: ProjectId,
}

impl<S: Send + Sync> FromRequestParts<S> for Project {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(project) = Path::<Project>::from_request_parts(parts, state)
            .await
            .map_err(|err| err.into_response())?;

        Ok(project)
    }
}

#[derive(Deserialize, Clone)]
pub struct Tenant {
    pub tenant: TenantId,
}

impl<S: Send + Sync> FromRequestParts<S> for Tenant {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(tenant_information) = Path::<Tenant>::from_request_parts(parts, state)
            .await
            .map_err(|err| err.into_response())?;

        Ok(tenant_information)
    }
}
