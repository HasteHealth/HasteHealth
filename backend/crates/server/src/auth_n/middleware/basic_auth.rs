use crate::{
    auth_n::oidc::{
        error::{OIDCError, OIDCErrorCode},
        routes::token::{
            ClientCredentialsMethod, TOKEN_EXPIRATION, client_credentials_to_token_response,
        },
        schemas::token_body::{OAuth2TokenBody, OAuth2TokenBodyGrantType},
    },
    extract::{
        basic_credentials::BasicCredentialsHeader,
        path_tenant::{ProjectIdentifier, TenantIdentifier},
    },
    services::AppState,
};
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::Cached;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, TenantId};
use haste_repository::Repository;

use std::{
    sync::{Arc, LazyLock},
    time::Duration,
};

#[derive(Hash, PartialEq, Eq)]
struct CacheTokenKey(String);
impl CacheTokenKey {
    fn new(tenant: &TenantId, project: &ProjectId, client_id: &str, client_secret: &str) -> Self {
        Self(format!(
            "{}:{}:{}:{}",
            tenant, project, client_id, client_secret
        ))
    }
}

// Token creation is expensive so caching for performance.
static CACHED_BASIC_TOKENS: LazyLock<
    // Tenant, Project, ClientId, ClientSecret
    moka::future::Cache<CacheTokenKey, String>,
> = LazyLock::new(|| {
    moka::future::Cache::builder()
        // Set as slightly less than the token expiration to ensure tokens are refreshed before they expire.
        .time_to_live(Duration::from_secs(TOKEN_EXPIRATION as u64 - 500))
        .build()
});

pub async fn basic_auth_middleware<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    Cached(ProjectIdentifier { project }): Cached<ProjectIdentifier>,
    State(state): State<Arc<AppState<Repo, Search, Terminology>>>,
    // run the `HeaderMap` extractor
    BasicCredentialsHeader(credentials): BasicCredentialsHeader,
    // you can also add more extractors here but the last
    // extractor must implement `FromRequest` which
    // `Request` does
    mut request: Request,
    next: Next,
) -> Result<Response, OIDCError> {
    if let Some(credentials) = credentials {
        if let Some(cached_token) = CACHED_BASIC_TOKENS
            .get(&CacheTokenKey::new(
                &tenant,
                &project,
                &credentials.0,
                &credentials.1,
            ))
            .await
        {
            request.headers_mut().insert(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", cached_token).parse().unwrap(),
            );
        } else {
            let res = client_credentials_to_token_response(
                state.as_ref(),
                &tenant,
                &project,
                &None,
                &OAuth2TokenBody {
                    client_id: credentials.0.clone(),
                    client_secret: Some(credentials.1.clone()),
                    code: None,
                    code_verifier: None,
                    grant_type: OAuth2TokenBodyGrantType::ClientCredentials,
                    redirect_uri: None,
                    refresh_token: None,
                    scope: None,
                },
                ClientCredentialsMethod::BasicAuth,
            )
            .await?;

            let Some(id_token) = res.id_token else {
                return Err(OIDCError::new(
                    OIDCErrorCode::AccessDenied,
                    Some("Failed to authorize client.".to_string()),
                    None,
                ));
            };

            CACHED_BASIC_TOKENS
                .insert(
                    CacheTokenKey::new(&tenant, &project, &credentials.0, &credentials.1),
                    id_token.clone(),
                )
                .await;

            request.headers_mut().insert(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", id_token).parse().unwrap(),
            );
        }
    }

    Ok(next.run(request).await)
}
