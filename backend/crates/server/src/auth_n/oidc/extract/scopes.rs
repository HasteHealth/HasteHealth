use crate::auth_n::oidc::middleware::OIDCParameters;
use axum::{
    Extension, RequestPartsExt,
    extract::FromRequestParts,
    http::request::Parts,
    response::{IntoResponse, Response},
};
use std::borrow::Cow;

pub struct Scopes(pub oxidized_jwt::scopes::Scopes);

impl<S: Send + Sync> FromRequestParts<S> for Scopes {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let Extension(oidc_params) = parts
            .extract::<Extension<OIDCParameters>>()
            .await
            .map_err(|err| err.into_response())?;

        let scope = oidc_params
            .parameters
            .get("scope")
            .map(|s| Cow::Borrowed(s))
            .unwrap_or_else(|| Cow::Owned("".to_string()));

        let scopes = oxidized_jwt::scopes::Scopes::try_from(scope.as_str())
            .map_err(|err| err.into_response())?;

        Ok(Scopes(scopes))
    }
}
