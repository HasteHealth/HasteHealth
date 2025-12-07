use crate::{
    auth_n::oidc::{
        error::{OIDCError, OIDCErrorCode},
        routes::token::client_credentials_to_token_response,
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
use haste_repository::Repository;
use std::sync::Arc;

pub async fn basic_auth<
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
        let res = client_credentials_to_token_response(
            state.as_ref(),
            &tenant,
            &project,
            &None,
            &OAuth2TokenBody {
                client_id: credentials.0,
                client_secret: Some(credentials.1),
                code: None,
                code_verifier: None,
                grant_type: OAuth2TokenBodyGrantType::ClientCredentials,
                redirect_uri: None,
                refresh_token: None,
                scope: None,
            },
        )
        .await?;

        if let Some(token_response) = res.id_token {
            request.headers_mut().insert(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", token_response).parse().unwrap(),
            );
        } else {
            return Err(OIDCError::new(
                OIDCErrorCode::AccessDenied,
                Some("Failed to authorize client.".to_string()),
                None,
            ));
        }
    }

    Ok(next.run(request).await)
}
