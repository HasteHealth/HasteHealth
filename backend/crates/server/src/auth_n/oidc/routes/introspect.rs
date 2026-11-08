//! Token introspection (RFC 7662), with the response fields SMART App Launch
//! adds (<https://hl7.org/fhir/smart-app-launch/STU2.2/token-introspection.html>).

use crate::{
    auth_n::{
        middleware::jwt::validate_jwt,
        oidc::{
            error::{OIDCError, OIDCErrorCode},
            extract::{body::OAuthIntrospectionBody, client_app::find_client_app},
            routes::token::{
                is_patient_bound_scope, require_client_secret, smart_style_url,
                verify_client_credentials,
            },
            schemas::token_introspection::OAuth2TokenIntrospectionBody,
        },
    },
    config::ServerConfig,
    extract::path_tenant::{ProjectIdentifier, TenantIdentifier},
    route_path::issuer_url,
    services::ServerState,
};
use axum::{Json, extract::State};
use axum_extra::{extract::Cached, routing::TypedPath};
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{
    ProjectId, TenantId,
    scopes::{LaunchSystemScope, Scope, SmartScope},
};
use haste_repository::Repository;
use serde::Serialize;
use std::sync::Arc;

#[derive(TypedPath)]
#[typed_path("/introspect")]
pub struct IntrospectPath;

/// An introspection response (RFC 7662 §2.2). An inactive token gets
/// `active: false` and nothing else, so every other field is optional.
#[derive(Serialize, Default, Debug)]
pub struct IntrospectionResponse {
    active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    token_type: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    exp: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sub: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    aud: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    iss: Option<String>,
    #[serde(rename = "fhirUser", skip_serializing_if = "Option::is_none")]
    fhir_user: Option<String>,
    // The SMART launch context the token response returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    patient: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    encounter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    need_patient_banner: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    smart_style_url: Option<String>,
}

fn invalid_client() -> OIDCError {
    OIDCError::new(
        OIDCErrorCode::InvalidClient,
        Some("Invalid credentials".to_string()),
        None,
    )
}

/// Requires the caller to authenticate as a confidential client of the
/// project, so the endpoint cannot be used to probe tokens (RFC 7662 §4).
async fn authenticate_caller<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    state: &ServerState<Repo, Search, Terminology>,
    tenant: &TenantId,
    project: &ProjectId,
    body: &OAuth2TokenIntrospectionBody,
) -> Result<(), OIDCError> {
    let Some(client_id) = body.client_id.as_deref() else {
        return Err(invalid_client());
    };

    // An unknown client gets the same answer as a wrong secret.
    let client_app = find_client_app(
        state,
        tenant.clone(),
        project.clone(),
        client_id.to_string(),
    )
    .await
    .map_err(|e| match e.error {
        OIDCErrorCode::InvalidClient => invalid_client(),
        _ => e,
    })?;

    require_client_secret(&client_app)?;
    verify_client_credentials(&client_app, Some(client_id), body.client_secret.as_deref())
}

/// Describes `token`. It is active when the project's FHIR API would accept
/// it; anything else (unknown, malformed, expired, or issued for another
/// project) is inactive.
fn introspect_token(
    config: &ServerConfig,
    tenant: &TenantId,
    project: &ProjectId,
    token: &str,
) -> Result<IntrospectionResponse, OIDCError> {
    let Ok(claims) = validate_jwt(config, tenant, project, token) else {
        return Ok(IntrospectionResponse::default());
    };

    if claims.tenant != *tenant || claims.project.as_ref() != Some(project) {
        return Ok(IntrospectionResponse::default());
    }

    // The token response returns the launch context with the `launch` scope,
    // which is only issued for an EHR launch, and the patient with the scopes
    // bound to it.
    let launch_granted = claims
        .scope
        .contains_scope(&Scope::SMART(SmartScope::LaunchSystem(LaunchSystemScope)));
    let patient_returned = launch_granted || claims.scope.0.iter().any(is_patient_bound_scope);

    let iss = issuer_url(&config.api_uri, tenant, project)
        .map_err(|e| OIDCError::new(OIDCErrorCode::ServerError, Some(e.to_string()), None))?;

    Ok(IntrospectionResponse {
        active: true,
        scope: Some(String::from(claims.scope)),
        client_id: claims.client_id,
        token_type: Some("Bearer"),
        exp: Some(claims.exp),
        sub: Some(claims.sub.to_string()),
        aud: Some(claims.aud),
        iss: Some(iss.to_string()),
        fhir_user: claims.fhir_user.and_then(|fhir_user| fhir_user.value),
        patient: claims.patient.filter(|_| patient_returned),
        encounter: claims.encounter,
        need_patient_banner: launch_granted.then_some(true),
        smart_style_url: if launch_granted {
            Some(smart_style_url(&config.api_uri, tenant, project)?)
        } else {
            None
        },
    })
}

pub async fn introspect<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    _: IntrospectPath,
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    Cached(ProjectIdentifier { project }): Cached<ProjectIdentifier>,
    State(state): State<Arc<ServerState<Repo, Search, Terminology>>>,
    OAuthIntrospectionBody(body): OAuthIntrospectionBody,
) -> Result<Json<IntrospectionResponse>, OIDCError> {
    authenticate_caller(&state, &tenant, &project, &body).await?;

    Ok(Json(introspect_token(
        state.config.as_ref(),
        &tenant,
        &project,
        &body.token,
    )?))
}
