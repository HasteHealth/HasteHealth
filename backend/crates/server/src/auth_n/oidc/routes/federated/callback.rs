use axum::{
    extract::{OriginalUri, Query, State},
    response::Redirect,
};
use axum_extra::{extract::Cached, routing::TypedPath};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use jsonwebtoken::DecodingKey;
use oxidized_fhir_client::FHIRClient;
use oxidized_fhir_model::r4::generated::{
    resources::{IdentityProvider, Resource, ResourceType, User},
    terminology::{IssueType, UserRole},
    types::{FHIRString, Reference},
};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_jwt::{ProjectId, TenantId};
use oxidized_repository::{Repository, admin::TenantAuthAdmin, types::user::CreateUser};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::sync::Arc;
use tower_sessions::Session;
use url::Url;

use crate::{
    ServerEnvironmentVariables,
    auth_n::{
        oidc::routes::federated::initiate::{get_idp_session_info, validate_and_get_idp},
        session,
    },
    extract::path_tenant::{Project, ProjectIdentifier, TenantIdentifier},
    fhir_client::{FHIRServerClient, ServerCTX},
    services::AppState,
};

#[derive(TypedPath, Deserialize)]
#[typed_path("/federated/{identity_provider_id}/callback")]
pub struct FederatedInitiate {
    pub identity_provider_id: String,
}

#[derive(Serialize)]
enum GrantType {
    #[serde(rename = "authorization_code")]
    AuthorizationCode,
}

#[derive(Serialize)]
struct FederatedTokenBodyRequest {
    pub grant_type: GrantType,
    pub code: String,
    pub redirect_uri: String,
    pub client_id: String,
    pub client_secret: Option<String>,
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

    tracing::info!("Decode key {:?}", jwk);

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

pub async fn create_user_if_not_exists<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    fhir_client: &Arc<FHIRServerClient<Repo, Search, Terminology>>,
    tenant: &TenantId,
    _project: &ProjectId,
    idp: &IdentityProvider,
    sub_claim: &str,
) -> Result<oxidized_fhir_model::r4::generated::resources::User, OperationOutcomeError> {
    let user_id = user_federated_id(idp, sub_claim)?;

    let system_ctx = Arc::new(ServerCTX::system(
        tenant.clone(),
        ProjectId::System,
        fhir_client.clone(),
    ));

    if let Some(Resource::User(user)) = fhir_client
        .read(system_ctx.clone(), ResourceType::User, user_id.clone())
        .await?
    {
        Ok(user)
    } else {
        let new_user = User {
            id: Some(user_id.clone()),
            role: Box::new(UserRole::Member(None)),
            federated: Some(Box::new(Reference {
                reference: Some(Box::new(FHIRString {
                    value: Some(format!("IdentityProvider/{}", idp.id.as_ref().unwrap())),
                    ..Default::default()
                })),
                ..Default::default()
            })),
            ..Default::default()
        };

        let created_user = fhir_client
            .update(
                system_ctx.clone(),
                ResourceType::User,
                user_id,
                Resource::User(new_user),
            )
            .await?;

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
    uri: OriginalUri,
    Query(CallbackQueryParams { code, state }): Query<CallbackQueryParams>,
    State(app_state): State<Arc<AppState<Repo, Search, Terminology>>>,
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    Cached(ProjectIdentifier { project }): Cached<ProjectIdentifier>,
    Cached(Project(project_resource)): Cached<Project>,
    Cached(session): Cached<Session>,
) -> Result<Redirect, OperationOutcomeError> {
    let identity_provider = validate_and_get_idp(
        &tenant,
        app_state.fhir_client.clone(),
        &project_resource,
        identity_provider_id.clone(),
    )
    .await?;

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

    let idp_session_info = get_idp_session_info(&session, &identity_provider).await?;

    if state != idp_session_info.state {
        return Err(OperationOutcomeError::error(
            IssueType::Invalid(None),
            "State parameter does not match the stored session state.".to_string(),
        ));
    }

    let federated_token_body = FederatedTokenBodyRequest {
        grant_type: GrantType::AuthorizationCode,
        code: code,
        redirect_uri: create_federated_callback_url(
            &app_state.config.get(ServerEnvironmentVariables::APIURI)?,
            &uri,
            &identity_provider_id,
            &FederatedInitiate {
                identity_provider_id: identity_provider_id.clone(),
            }
            .to_string(),
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
            OperationOutcomeError::error(
                IssueType::Invalid(None),
                "Failed at sending request to identity provider token endpoint".to_string(),
            )
        })?;

    let token_response_body = res
        .json::<FederatedTokenBodyResponse>()
        .await
        .map_err(|_e| {
            OperationOutcomeError::error(
                IssueType::Invalid(None),
                "Failed to parse token response from identity provider".to_string(),
            )
        })?;

    let id_token = token_response_body.id_token;

    let claims = decode_using_jwk(&id_token, &jwk_url).await?;

    let user = create_user_if_not_exists(
        &app_state.fhir_client,
        &tenant,
        &project,
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
    uri: &OriginalUri,
    idp_id: &str,
    replace_path: &str,
) -> Result<String, OperationOutcomeError> {
    let Ok(api_url) = Url::parse(&api_url_string) else {
        return Err(OperationOutcomeError::error(
            IssueType::Exception(None),
            "Invalid API_URL format".to_string(),
        ));
    };

    let path = uri.path().to_string().replace(
        replace_path,
        &FederatedInitiate {
            identity_provider_id: idp_id.to_string(),
        }
        .to_string(),
    );

    Ok(api_url.join(&path).unwrap().to_string())
}
