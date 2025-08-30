use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use jsonwebtoken::Validation;

use crate::{
    auth_n::{certificates, claims::TokenClaims},
    extract::bearer_token::AuthBearer,
};

fn validate_jwt(token: &str) -> Result<TokenClaims, StatusCode> {
    let result = jsonwebtoken::decode::<TokenClaims>(
        token,
        certificates::decoding_key(),
        &Validation::new(jsonwebtoken::Algorithm::RS256),
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
            request.extensions_mut().insert(claims);
            Ok(next.run(request).await)
        }
        Err(status_code) => Err(status_code),
    }
}
