use crate::{
    auth_n::certificates, extract::bearer_token::AuthBearer, route_path::project_path,
    services::AppState,
};
use axum::{
    extract::{OriginalUri, Request, State},
    http::{HeaderMap, StatusCode, Uri},
    middleware::Next,
    response::{IntoResponse as _, Response},
};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, TenantId, claims::UserTokenClaims};
use haste_repository::Repository;
use jsonwebtoken::Validation;
use std::{
    path::PathBuf,
    sync::{Arc, LazyLock},
};
use url::Url;

static VALIDATION_CONFIG: LazyLock<Validation> = LazyLock::new(|| {
    let mut config = Validation::new(jsonwebtoken::Algorithm::RS256);
    config.validate_aud = false;
    config
});

fn validate_jwt(token: &str) -> Result<UserTokenClaims, StatusCode> {
    let header = jsonwebtoken::decode_header(token).map_err(|_| StatusCode::UNAUTHORIZED)?;

    let cert_provider = certificates::get_certification_provider();

    let decoding_key = cert_provider
        .decoding_key(&header.kid.unwrap_or_else(|| "".to_string()).as_str())
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let result = jsonwebtoken::decode::<UserTokenClaims>(
        token,
        &decoding_key.decoding_key,
        &*VALIDATION_CONFIG,
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    Ok(result.claims)
}

pub fn derive_well_known_openid_configuration_url(
    api_url: &str,
    tenant: &TenantId,
    project: &ProjectId,
) -> Result<Url, OperationOutcomeError> {
    let path = PathBuf::from("/.well-known/openid-configuration");

    if let Ok(api_url) = Url::parse(&api_url) {
        api_url
            .join(
                path.join(project_path(tenant, project).strip_prefix("/").unwrap())
                    .to_str()
                    .unwrap_or_default(),
            )
            .map_err(|e| {
                tracing::error!("Failed to derive well-known URL: {:?}", e);
                OperationOutcomeError::error(
                    IssueType::Invalid(None),
                    "Invalid API URL configured".to_string(),
                )
            })
    } else {
        Err(OperationOutcomeError::error(
            IssueType::Invalid(None),
            "Invalid API URL configured".to_string(),
        ))
    }
}

pub fn derive_protected_resource_metadata_url(
    resource_uri: &Uri,
    api_url: &str,
) -> Result<Url, OperationOutcomeError> {
    let path = PathBuf::from("/.well-known/oauth-protected-resource");
    if let Ok(api_url) = Url::parse(&api_url) {
        let tenant_url = api_url
            .join(
                path.join(resource_uri.path().strip_prefix("/").unwrap_or_default())
                    .to_str()
                    .unwrap_or_default(),
            )
            .map_err(|e| {
                tracing::error!("Failed to derive well-known URL: {:?}", e);
                OperationOutcomeError::error(
                    IssueType::Invalid(None),
                    "Invalid API URL configured".to_string(),
                )
            })?;

        Ok(tenant_url)
    } else {
        Err(OperationOutcomeError::error(
            IssueType::Invalid(None),
            "Invalid API URL configured".to_string(),
        ))
    }
}

fn invalid_jwt_response(uri: &Uri, api_url: &str, status_code: StatusCode) -> Response {
    tracing::warn!(
        "Invalid JWT token provided in request sending '{}'",
        status_code
    );

    let Ok(protected_resource_metadata_url) = derive_protected_resource_metadata_url(uri, api_url)
    else {
        return (status_code).into_response();
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::WWW_AUTHENTICATE,
        format!(
            r#"Bearer resource_metadata="{}""#,
            protected_resource_metadata_url.to_string()
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
    State(state): State<Arc<AppState<Repo, Search, Terminology>>>,
    // run the `HeaderMap` extractor
    AuthBearer(token): AuthBearer,
    // you can also add more extractors here but the last
    // extractor must implement `FromRequest` which
    // `Request` does
    OriginalUri(uri): OriginalUri,
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    let Some(token) = token else {
        return Err(invalid_jwt_response(
            &uri,
            &state
                .config
                .get(crate::ServerEnvironmentVariables::APIURI)
                .unwrap_or_default(),
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
                &uri,
                &state
                    .config
                    .get(crate::ServerEnvironmentVariables::APIURI)
                    .unwrap_or_default(),
                status_code,
            )),
            _ => Err((status_code).into_response()),
        },
    }
}
