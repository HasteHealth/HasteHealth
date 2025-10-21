use axum::{
    extract::{FromRequestParts, Path},
    http::request::Parts,
    response::{IntoResponse, Response},
};
use oxidized_repository::types::{ProjectId, TenantId};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct ProjectIdentifier {
    pub project: ProjectId,
}

impl<S: Send + Sync> FromRequestParts<S> for ProjectIdentifier {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(project) = Path::<ProjectIdentifier>::from_request_parts(parts, state)
            .await
            .map_err(|err| err.into_response())?;

        Ok(project)
    }
}

#[derive(Deserialize, Clone)]
pub struct TenantIdentifier {
    pub tenant: TenantId,
}

impl<S: Send + Sync> FromRequestParts<S> for TenantIdentifier {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(tenant_information) = Path::<TenantIdentifier>::from_request_parts(parts, state)
            .await
            .map_err(|err| err.into_response())?;

        Ok(tenant_information)
    }
}
