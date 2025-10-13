use std::sync::{Arc, LazyLock};

use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use jsonwebtoken::Validation;

use crate::{
    auth_n::{certificates, claims::UserTokenClaims},
    extract::bearer_token::AuthBearer,
};

static VALIDATION_CONFIG: LazyLock<Validation> = LazyLock::new(|| {
    let mut config = Validation::new(jsonwebtoken::Algorithm::RS256);
    config.validate_aud = false;
    config
});

fn validate_jwt(token: &str) -> Result<UserTokenClaims, StatusCode> {
    let result = jsonwebtoken::decode::<UserTokenClaims>(
        token,
        certificates::decoding_key(),
        &*VALIDATION_CONFIG,
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    Ok(result.claims)
}

pub async fn token_verifcation(
    // run the `HeaderMap` extractor
    AuthBearer(token): AuthBearer,
    // you can also add more extractors here but the last
    // extractor must implement `FromRequest` which
    // `Request` does
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    match validate_jwt(&token) {
        Ok(claims) => {
            request.extensions_mut().insert(Arc::new(claims));
            Ok(next.run(request).await)
        }
        Err(status_code) => Err(status_code),
    }
}
