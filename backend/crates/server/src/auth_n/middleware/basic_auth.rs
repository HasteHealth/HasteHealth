use crate::{
    auth_n::oidc::{
        error::{OIDCError, OIDCErrorCode},
        routes::token::{ClientCredentialsMethod, client_credentials_to_token_response},
        schemas::token_body::{OAuth2TokenBody, OAuth2TokenBodyGrantType},
    },
    extract::{
        basic_credentials::BasicCredentialsHeader,
        path_tenant::{ProjectIdentifier, TenantIdentifier},
    },
    services::ServerState,
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

use sha2::{Digest, Sha256};
use std::{
    sync::{Arc, LazyLock},
    time::Duration,
};

/// How long a token minted for a Basic-auth credential is reused before the
/// credential is checked against the client application again.
const CACHED_TOKEN_TTL: Duration = Duration::from_secs(5 * 60);

#[derive(Hash, PartialEq, Eq)]
struct CacheTokenKey([u8; 32]);
impl CacheTokenKey {
    fn new(tenant: &TenantId, project: &ProjectId, client_id: &str, client_secret: &str) -> Self {
        let mut hasher = Sha256::new();
        // Length-prefixed so that no two different credentials can digest to the
        // same byte string by shifting characters across the field boundaries.
        for part in [tenant.as_ref(), project.as_ref(), client_id, client_secret] {
            hasher.update((part.len() as u64).to_be_bytes());
            hasher.update(part.as_bytes());
        }

        Self(hasher.finalize().into())
    }
}

// Token creation is expensive so caching for performance.
static CACHED_BASIC_TOKENS: LazyLock<moka::future::Cache<CacheTokenKey, String>> =
    LazyLock::new(|| {
        moka::future::Cache::builder()
            .time_to_live(CACHED_TOKEN_TTL)
            .build()
    });

pub async fn basic_auth_middleware<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    Cached(ProjectIdentifier { project }): Cached<ProjectIdentifier>,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
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
                    client_id: Some(credentials.0.clone()),
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
