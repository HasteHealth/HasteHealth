use axum::{
    extract::{Query, State},
    response::Redirect,
};
use axum_extra::{extract::Cached, routing::TypedPath};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use haste_fhir_client::FHIRClient;
use haste_fhir_model::r4::generated::{
    resources::{
        Bundle, BundleEntry, BundleEntryRequest, IdentityProvider, Membership, Resource,
        ResourceType, User,
    },
    terminology::{BundleType, HttpVerb, IssueType, UserRole},
    types::{FHIRString, FHIRUri, Reference},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, TenantId};
use haste_repository::{Repository, admin::TenantAuthAdmin, types::user::CreateUser};
use jsonwebtoken::DecodingKey;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::sync::Arc;
use tower_sessions::Session;
use url::Url;

use crate::{
    ServerEnvironmentVariables,
    auth_n::{
        oidc::routes::federated::initiate::{get_idp, get_idp_session_info},
        session,
    },
    extract::path_tenant::{ProjectIdentifier, TenantIdentifier},
    fhir_client::ServerCTX,
    services::AppState,
};

#[derive(TypedPath, Deserialize)]
#[typed_path("/federated/{identity_provider_id}/callback")]
pub struct FederatedInitiate {
    pub identity_provider_id: String,
}

#[derive(Serialize, Debug)]
enum GrantType {
    #[serde(rename = "authorization_code")]
    AuthorizationCode,
}

#[derive(Serialize, Debug)]
struct FederatedTokenBodyRequest {
    pub grant_type: GrantType,
    pub code: String,
    pub redirect_uri: String,
    pub client_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_verifier: Option<String>,
}

#[derive(Deserialize)]
struct FederatedTokenBodyResponse {
    // pub access_token: String,
    pub id_token: String,
}

#[derive(Deserialize)]
pub struct CallbackQueryParams {
    pub code: String,
    pub state: String,
}

#[derive(Deserialize)]
struct FederatedTokenClaims {
    pub sub: String,
}

async fn decode_using_jwk(
    token: &str,
    jwk_url: &str,
) -> Result<FederatedTokenClaims, OperationOutcomeError> {
    let header = jsonwebtoken::decode_header(token).map_err(|_| {
        OperationOutcomeError::error(
            IssueType::Invalid(None),
            "Failed to decode token header".to_string(),
        )
    })?;

    let res = reqwest::get(jwk_url).await.map_err(|_e| {
        OperationOutcomeError::error(
            IssueType::Invalid(None),
            "Failed to fetch JWKs from identity provider".to_string(),
        )
    })?;

    let jwk_set = res
        .json::<jsonwebtoken::jwk::JwkSet>()
        .await
        .map_err(|_e| {
            OperationOutcomeError::error(
                IssueType::Invalid(None),
                "Failed to parse JWKs from identity provider".to_string(),
            )
        })?;

    let jwk = if let Some(kid) = header.kid.as_ref() {
        jwk_set.find(kid)
    } else {
        jwk_set.keys.first()
    };

    let jwk = jwk.ok_or_else(|| {
        OperationOutcomeError::error(
            IssueType::Invalid(None),
            "No matching JWK found for token".to_string(),
        )
    })?;

    let decoding_key = DecodingKey::from_jwk(&jwk).map_err(|_e| {
        OperationOutcomeError::error(
            IssueType::Invalid(None),
            "Failed to create decoding key from JWK".to_string(),
        )
    })?;

    let mut token_validation_settings = jsonwebtoken::Validation::new(header.alg);
    token_validation_settings.validate_aud = false;

    let result = jsonwebtoken::decode::<FederatedTokenClaims>(
        token,
        &decoding_key,
        &token_validation_settings,
    )
    .map_err(|e| {
        tracing::error!("Federated token decode error: {:?}", e);

        OperationOutcomeError::error(
            IssueType::Invalid(None),
            "Failed to decode and verify token. Ensure openid is in scope and claims contain a sub claim.".to_string(),
        )
    })?;

    Ok(result.claims)
}

fn user_federated_id(idp: &IdentityProvider, sub: &str) -> Result<String, OperationOutcomeError> {
    let Some(id_prefix) = idp.id.as_ref() else {
        return Err(OperationOutcomeError::error(
            IssueType::Invalid(None),
            "Identity Provider is missing ID".to_string(),
        ));
    };

    let mut sha_hasher = Sha1::new();
    sha_hasher.update(sub.as_bytes());
    let hashed_user_sub_claim = URL_SAFE_NO_PAD.encode(&sha_hasher.finalize());

    Ok(format!("{}|{}", id_prefix, hashed_user_sub_claim))
}

async fn create_user_if_not_exists<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    app_state: &Arc<AppState<Repo, Search, Terminology>>,
    tenant: &TenantId,
    target_project: &ProjectId,
    idp: &IdentityProvider,
    sub_claim: &str,
) -> Result<haste_fhir_model::r4::generated::resources::User, OperationOutcomeError> {
    let user_id = user_federated_id(idp, sub_claim)?;

    let mut existing_user = app_state
        .fhir_client
        .batch(
            Arc::new(ServerCTX::system(
                tenant.clone(),
                target_project.clone(),
                app_state.fhir_client.clone(),
            )),
            Bundle {
                type_: Box::new(BundleType::Batch(None)),
                entry: Some(vec![
                    BundleEntry {
                        request: Some(BundleEntryRequest {
                            method: Box::new(HttpVerb::GET(None)),
                            url: Box::new(FHIRUri {
                                value: Some(format!("{}/{}", ResourceType::User.as_ref(), user_id)),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                    // Membership search for linked user
                    BundleEntry {
                        request: Some(BundleEntryRequest {
                            method: Box::new(HttpVerb::GET(None)),
                            url: Box::new(FHIRUri {
                                value: Some(format!(
                                    "{}?user={}/{}&_count=1",
                                    ResourceType::Membership.as_ref(),
                                    ResourceType::User.as_ref(),
                                    user_id
                                )),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                ]),

                ..Default::default()
            },
        )
        .await?;

    if let Some(Resource::User(user)) = existing_user
        .entry
        .as_mut()
        .and_then(|entries| entries.pop())
        .and_then(|e| e.resource)
        .map(|r| *r)
        && let Some(Resource::Membership(_project_membership)) = existing_user
            .entry
            .as_ref()
            .and_then(|entries| entries.get(0))
            .and_then(|e| e.resource.as_ref())
            .and_then(|r| match r.as_ref() {
                Resource::Bundle(bundle) => bundle
                    .entry
                    .as_ref()
                    .and_then(|entries| entries.get(0))
                    .and_then(|e| e.resource.as_ref())
                    .and_then(|r| Some(r.as_ref())),
                _ => None,
            })
    {
        Ok(user)
    } else {
        let transaction = app_state.transaction().await?;
        // Need to create both User and Membership resources.
        // User resource will exist on system project, Membership on target project.
        let created_user = {
            let user = transaction
                .fhir_client
                .update(
                    Arc::new(ServerCTX::system(
                        tenant.clone(),
                        ProjectId::System,
                        transaction.fhir_client.clone(),
                    )),
                    ResourceType::User,
                    user_id.clone(),
                    Resource::User(User {
                        id: Some(user_id.clone()),
                        role: Box::new(UserRole::Member(None)),
                        federated: Some(Box::new(Reference {
                            reference: Some(Box::new(FHIRString {
                                value: Some(format!(
                                    "{}/{}",
                                    ResourceType::IdentityProvider.as_ref(),
                                    idp.id.as_ref().unwrap()
                                )),
                                ..Default::default()
                            })),
                            ..Default::default()
                        })),
                        ..Default::default()
                    }),
                )
                .await?;

            transaction
                .fhir_client
                .update(
                    Arc::new(ServerCTX::system(
                        tenant.clone(),
                        target_project.clone(),
                        transaction.fhir_client.clone(),
                    )),
                    ResourceType::Membership,
                    user_id.clone(),
                    Resource::Membership(Membership {
                        id: Some(user_id.clone()),
                        user: Box::new(Reference {
                            reference: Some(Box::new(FHIRString {
                                value: Some(format!(
                                    "{}/{}",
                                    ResourceType::User.as_ref(),
                                    user_id.clone()
                                )),
                                ..Default::default()
                            })),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                )
                .await?;

            user
        };

        transaction.commit().await?;

        Ok(match created_user {
            Resource::User(user) => user,
            _ => {
                return Err(OperationOutcomeError::error(
                    IssueType::Exception(None),
                    "Failed to create federated user".to_string(),
                ));
            }
        })
    }
}

pub async fn federated_callback<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    FederatedInitiate {
        identity_provider_id,
    }: FederatedInitiate,
    Query(CallbackQueryParams { code, state }): Query<CallbackQueryParams>,
    State(app_state): State<Arc<AppState<Repo, Search, Terminology>>>,
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    Cached(ProjectIdentifier { project }): Cached<ProjectIdentifier>,
    Cached(session): Cached<Session>,
) -> Result<Redirect, OperationOutcomeError> {
    let identity_provider = get_idp(
        &tenant,
        app_state.fhir_client.clone(),
        identity_provider_id.clone(),
    )
    .await?;

    let idp_session_info = get_idp_session_info(&session, &identity_provider).await?;

    let client_id = identity_provider
        .oidc
        .as_ref()
        .map(|oidc| oidc.client.clientId.as_ref())
        .and_then(|c| c.value.as_ref())
        .ok_or_else(|| {
            OperationOutcomeError::error(
                IssueType::Invalid(None),
                "Identity Provider is missing client ID".to_string(),
            )
        })?;

    let client_secret = identity_provider
        .oidc
        .as_ref()
        .and_then(|oidc| oidc.client.secret.as_ref())
        .and_then(|secret| secret.value.as_ref());

    if state != idp_session_info.state {
        return Err(OperationOutcomeError::error(
            IssueType::Invalid(None),
            "State parameter does not match the stored session state.".to_string(),
        ));
    }

    if project != ProjectId::System && idp_session_info.project != project {
        return Err(OperationOutcomeError::error(
            IssueType::Invalid(None),
            "Project in session does not match the current project.".to_string(),
        ));
    }

    let federated_token_body = FederatedTokenBodyRequest {
        grant_type: GrantType::AuthorizationCode,
        code: code,
        redirect_uri: create_federated_callback_url(
            &app_state.config.get(ServerEnvironmentVariables::APIURI)?,
            &tenant,
            &identity_provider_id,
        )?,
        client_id: client_id.clone(),
        client_secret: client_secret.cloned(),
        code_verifier: idp_session_info.code_verifier,
    };

    let token_url = identity_provider
        .oidc
        .as_ref()
        .map(|oidc| &oidc.token_endpoint)
        .and_then(|uri| uri.value.as_ref())
        .ok_or_else(|| {
            OperationOutcomeError::error(
                IssueType::Invalid(None),
                "Identity Provider is missing token endpoint".to_string(),
            )
        })?;

    let jwk_url = identity_provider
        .oidc
        .as_ref()
        .and_then(|oidc| oidc.jwks_uri.as_ref())
        .and_then(|uri| uri.value.as_ref())
        .ok_or_else(|| {
            OperationOutcomeError::error(
                IssueType::Invalid(None),
                "Identity Provider is missing JWKS URI".to_string(),
            )
        })?;

    let client = reqwest::Client::new();
    let res = client
        .post(token_url)
        .form(&federated_token_body)
        .send()
        .await
        .map_err(|_e| {
            tracing::error!("Failed to send request to token endpoint: {:?}", _e);

            OperationOutcomeError::error(
                IssueType::Invalid(None),
                "Failed at sending request to identity provider token endpoint".to_string(),
            )
        })?;

    if !res.status().is_success() {
        let status = res.status();
        tracing::error!(
            "Token endpoint returned: '{}'",
            res.text().await.unwrap_or_default()
        );
        tracing::error!("Token endpoint returned error status: {}", status);
        return Err(OperationOutcomeError::error(
            IssueType::Invalid(None),
            format!(
                "Identity provider token endpoint returned error status: {}",
                status
            ),
        ));
    }

    let token_response_body = res
        .json::<FederatedTokenBodyResponse>()
        .await
        .map_err(|_e| {
            tracing::error!("Failed to parse token response body: {:?}", _e);

            OperationOutcomeError::error(
                IssueType::Invalid(None),
                "Failed to parse token response from identity provider".to_string(),
            )
        })?;

    let id_token = token_response_body.id_token;

    let claims = decode_using_jwk(&id_token, &jwk_url).await?;

    let user = create_user_if_not_exists(
        &app_state,
        &tenant,
        &idp_session_info.project,
        &identity_provider,
        &claims.sub,
    )
    .await?;

    let Some(user_model) = TenantAuthAdmin::<CreateUser, _, _, _, _>::read(
        app_state.repo.as_ref(),
        &tenant,
        &user.id.unwrap(),
    )
    .await?
    else {
        return Err(OperationOutcomeError::error(
            IssueType::Exception(None),
            "Failed to retrieve created federated user from repository".to_string(),
        ));
    };

    session::user::set_user(&session, &tenant, &user_model).await?;

    // Will redirect authorize_path
    Ok(Redirect::to(&idp_session_info.redirect_to))
}

pub fn create_federated_callback_url(
    api_url_string: &str,
    tenant: &TenantId,
    idp_id: &str,
) -> Result<String, OperationOutcomeError> {
    let Ok(api_url) = Url::parse(&api_url_string) else {
        return Err(OperationOutcomeError::error(
            IssueType::Exception(None),
            "Invalid API_URL format".to_string(),
        ));
    };

    Ok(api_url
        .join(&format!(
            "w/{}/api/v1/system/oidc/federated/{}/callback",
            tenant.as_ref(),
            idp_id
        ))
        .unwrap()
        .to_string())
}
