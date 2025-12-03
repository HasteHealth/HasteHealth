use crate::{
    auth_n::{certificates, oidc::routes::discovery::WellKnown},
    extract::{
        bearer_token::AuthBearer,
        path_tenant::{ProjectIdentifier, TenantIdentifier},
    },
    services::AppState,
};
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse as _, Response},
};
use axum_extra::extract::Cached;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, TenantId, claims::UserTokenClaims};
use haste_repository::Repository;
use jsonwebtoken::Validation;
use std::sync::{Arc, LazyLock};
use url::Url;

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

fn invalid_jwt_response(
    api_url: &str,
    tenant: &TenantId,
    project: &ProjectId,
    status_code: StatusCode,
) -> Response {
    let well_known = WellKnown.to_string();
    let Ok(api_url) = Url::parse(&api_url) else {
        return (status_code).into_response();
    };

    let Ok(well_known_url) = api_url.join(&format!(
        "/w/{}/api/v1/{}/oidc{}",
        tenant.as_ref(),
        project.as_ref(),
        well_known
    )) else {
        return (status_code).into_response();
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::WWW_AUTHENTICATE,
        format!(
            r#"Bearer resource_metadata="{}""#,
            well_known_url.to_string()
        )
        .parse()
        .unwrap(),
    );
    (status_code, headers).into_response()
}

pub async fn token_verifcation<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    Cached(ProjectIdentifier { project }): Cached<ProjectIdentifier>,
    State(state): State<Arc<AppState<Repo, Search, Terminology>>>,
    // run the `HeaderMap` extractor
    AuthBearer(token): AuthBearer,
    // you can also add more extractors here but the last
    // extractor must implement `FromRequest` which
    // `Request` does
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    let Some(token) = token else {
        return Err(invalid_jwt_response(
            &state
                .config
                .get(crate::ServerEnvironmentVariables::APIURI)
                .unwrap_or_default(),
            &tenant,
            &project,
            StatusCode::UNAUTHORIZED,
        ));
    };

    match validate_jwt(&token) {
        Ok(claims) => {
            request.extensions_mut().insert(Arc::new(claims));
            Ok(next.run(request).await)
        }
        Err(status_code) => match status_code {
            StatusCode::UNAUTHORIZED => Err(invalid_jwt_response(
                &state
                    .config
                    .get(crate::ServerEnvironmentVariables::APIURI)
                    .unwrap_or_default(),
                &tenant,
                &project,
                status_code,
            )),
            _ => Err((status_code).into_response()),
        },
    }
}
