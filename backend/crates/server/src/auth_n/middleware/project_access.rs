use axum::{Extension, extract::Request, http::StatusCode, middleware::Next, response::Response};

use crate::{
    auth_n::claims::UserTokenClaims,
    extract::path_tenant::{Project, Tenant},
};

pub async fn project_access(
    Tenant { tenant }: Tenant,
    Project { project }: Project,
    // run the `HeaderMap` extractor
    Extension(claims): Extension<UserTokenClaims>,
    // you can also add more extractors here but the last
    // extractor must implement `FromRequest` which
    // `Request` does
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if claims.tenant != tenant {
        return Err(StatusCode::FORBIDDEN);
    }

    if let Some(user_project) = &claims.project {
        if user_project != &project {
            return Err(StatusCode::FORBIDDEN);
        }
    } else {
        return Err(StatusCode::FORBIDDEN);
    }

    request.extensions_mut().insert(claims);
    Ok(next.run(request).await)
}
