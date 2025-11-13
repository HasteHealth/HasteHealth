use crate::{
    ServerEnvironmentVariables,
    auth_n::oidc::{
        code_verification::{generate_code_challenge, generate_code_verifier},
        extract::client_app::OIDCClientApplication,
        routes::federated::callback::federated_callback_url,
    },
    extract::path_tenant::{Project, TenantIdentifier},
    fhir_client::ServerCTX,
    services::AppState,
};
use axum::{
    extract::{OriginalUri, State},
    response::Redirect,
};
use axum_extra::{extract::Cached, routing::TypedPath};
use oxidized_fhir_client::FHIRClient;
use oxidized_fhir_model::r4::generated::{
    resources::{IdentityProvider, Project as FHIRProject, Resource, ResourceType},
    terminology::{IdentityProviderPkceChallengeMethod, IssueType},
};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_jwt::ProjectId;
use oxidized_repository::{
    Repository, types::authorization_code::PKCECodeChallengeMethod, utilities::generate_id,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_sessions::Session;
use url::Url;

#[derive(TypedPath, Deserialize)]
#[typed_path("/federated/{identity_provider_id}/initiate")]
pub struct FederatedInitiate {
    pub identity_provider_id: String,
}

fn validate_identity_provider_in_project(
    identity_provider_id: &str,
    project: &FHIRProject,
) -> Result<(), OperationOutcomeError> {
    if let Some(identity_providers) = &project.identityProvider {
        for ip_ref in identity_providers {
            if let Some(ref_id) = &ip_ref.reference.as_ref().and_then(|r| r.value.as_ref()) {
                if ref_id.as_str() == &format!("IdentityProvider/{}", identity_provider_id) {
                    return Ok(());
                }
            }
        }
    }
    Err(OperationOutcomeError::error(
        IssueType::Forbidden(None),
        "The specified identity provider is not associated with the project.".to_string(),
    ))
}

#[derive(Deserialize, Serialize, Clone)]
pub struct IDPSessionInfo {
    state: String,
    redirect_to: String,
    code_verifier: Option<String>,
}

async fn set_session_info(
    session: &mut Session,
    idp: &IdentityProvider,
    redirect_to: &str,
) -> Result<IDPSessionInfo, OperationOutcomeError> {
    let idp_id = idp.id.as_ref().ok_or_else(|| {
        OperationOutcomeError::error(
            IssueType::Invalid(None),
            "Identity Provider resource is missing an ID.".to_string(),
        )
    })?;

    let state = generate_id(Some(20));

    let mut info = IDPSessionInfo {
        state,
        redirect_to: redirect_to.to_string(),
        code_verifier: None,
    };

    if let Some(oidc) = &idp.oidc {
        if let Some(pkce) = &oidc.pkce {
            if pkce.enabled.as_ref().and_then(|b| b.value).unwrap_or(false) {
                let code_verifier = generate_code_verifier();
                info.code_verifier = Some(code_verifier);
            }
        }
    }

    session
        .insert(&format!("federated_initiate_{}", idp_id), &info)
        .await
        .map_err(|_| {
            OperationOutcomeError::error(
                IssueType::Exception(None),
                "Failed to set session information.".to_string(),
            )
        })?;

    Ok(info)
}

fn oidc_pkce_challenge_method(
    challenge: &IdentityProviderPkceChallengeMethod,
) -> Option<PKCECodeChallengeMethod> {
    match challenge {
        IdentityProviderPkceChallengeMethod::S256(None) => Some(PKCECodeChallengeMethod::S256),
        IdentityProviderPkceChallengeMethod::Plain(None) => Some(PKCECodeChallengeMethod::Plain),
        _ => None,
    }
}

async fn create_federated_authorization_url(
    session: &mut Session,
    api_uri: &str,
    original_uri: &OriginalUri,
    identity_provider: &IdentityProvider,
) -> Result<Url, OperationOutcomeError> {
    if let Some(oidc) = &identity_provider.oidc {
        let mut authorization_url = oidc
            .authorization_endpoint
            .value
            .as_ref()
            .and_then(|s| Url::parse(s).ok())
            .ok_or_else(|| {
                OperationOutcomeError::error(
                    IssueType::Invalid(None),
                    "Invalid authorization endpoint URL for identity provider".to_string(),
                )
            })?;

        let client_id = oidc.client.clientId.value.as_ref().ok_or_else(|| {
            OperationOutcomeError::error(
                IssueType::Invalid(None),
                "Missing client ID for identity provider.".to_string(),
            )
        })?;

        let scopes = oidc.scopes.as_ref().map(|s| {
            s.iter()
                .filter_map(|v| v.value.as_ref())
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(" ")
        });

        authorization_url.set_query(Some("response_type=code"));
        authorization_url
            .query_pairs_mut()
            .append_pair("client_id", client_id)
            .append_pair("scope", &scopes.unwrap_or_default())
            .append_pair(
                "redirect_uri",
                &federated_callback_url(
                    api_uri,
                    original_uri,
                    &identity_provider.id.clone().unwrap_or_default(),
                    &(FederatedInitiate {
                        identity_provider_id: identity_provider.id.clone().unwrap_or_default(),
                    }
                    .to_string()),
                )?,
            );

        let info = set_session_info(session, &identity_provider, "").await?;
        authorization_url
            .query_pairs_mut()
            .append_pair("state", &info.state);
        if let Some(code_verifier) = info.code_verifier
            && let Some(challenge_method) = oidc
                .pkce
                .as_ref()
                .and_then(|p| p.code_challenge_method.as_ref())
                .and_then(|c| oidc_pkce_challenge_method(c))
        {
            let code_challenge = generate_code_challenge(&code_verifier, &challenge_method)?;
            authorization_url
                .query_pairs_mut()
                .append_pair("code_challenge", &code_challenge);
            authorization_url
                .query_pairs_mut()
                .append_pair("code_challenge_method", &String::from(challenge_method));
        }

        Ok(authorization_url)
    } else {
        return Err(OperationOutcomeError::error(
            IssueType::NotFound(None),
            "The specified identity provider was not found.".to_string(),
        ));
    }
}

pub async fn federated_initiate<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    FederatedInitiate {
        identity_provider_id,
    }: FederatedInitiate,
    Cached(mut current_session): Cached<Session>,
    uri: OriginalUri,
    State(state): State<Arc<AppState<Repo, Search, Terminology>>>,
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    Cached(Project(project_resource)): Cached<Project>,
    OIDCClientApplication(_client_app): OIDCClientApplication,
    _uri: OriginalUri,
) -> Result<Redirect, OperationOutcomeError> {
    let api_uri = state.config.get(ServerEnvironmentVariables::APIURI)?;
    validate_identity_provider_in_project(&identity_provider_id, &project_resource)?;
    let identity_provider = state
        .fhir_client
        .read(
            Arc::new(ServerCTX::system(
                tenant,
                ProjectId::System,
                state.fhir_client.clone(),
            )),
            ResourceType::IdentityProvider,
            identity_provider_id,
        )
        .await?
        .and_then(|r| match r {
            Resource::IdentityProvider(ip) => Some(ip),
            _ => None,
        })
        .ok_or_else(|| {
            OperationOutcomeError::error(
                IssueType::NotFound(None),
                "The specified identity provider was not found.".to_string(),
            )
        })?;

    let federated_authorization_url = create_federated_authorization_url(
        &mut current_session,
        &api_uri,
        &uri,
        &identity_provider,
    )
    .await?;

    Ok(Redirect::to(federated_authorization_url.as_str()))
}
