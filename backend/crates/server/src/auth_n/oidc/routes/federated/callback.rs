use axum::{
    extract::{Query, State},
    response::Redirect,
};
use axum_extra::{extract::Cached, routing::TypedPath};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use haste_fhir_client::FHIRClient;
use haste_fhir_model::r4::generated::{
    resources::{
        AccessPolicyV2Assignment, Bundle, BundleEntry, BundleEntryRequest, IdentityProvider,
        Membership, Resource, ResourceType, User,
    },
    terminology::{BundleType, HttpVerb, IssueType, UserRole},
    types::{FHIRBoolean, FHIRString, FHIRUri, HumanName, Reference},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, TenantId};
use haste_repository::{Repository, admin::TenantModelAdmin, types::user::CreateUser};
use jsonwebtoken::DecodingKey;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::sync::Arc;
use tower_sessions::Session;
use url::Url;

use crate::{
    auth_n::{
        mfa::routes::totp_verification::totp_verification_route,
        oidc::routes::federated::initiate::{get_idp, get_idp_session_info},
        session::{self, user::SessionAuthorizationState},
    },
    extract::path_tenant::{ProjectIdentifier, TenantIdentifier},
    fhir_client::ServerCTX,
    services::ServerState,
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
    /// Absent when the authorization request was not an OIDC one, which is
    /// what a provider configured without the `openid` scope returns.
    pub id_token: Option<String>,
}

#[derive(Deserialize)]
pub struct CallbackQueryParams {
    pub code: Option<String>,
    pub state: Option<String>,
    /// Set instead of `code` when the provider rejects the authorization
    /// request, for instance because the client is not allowed a requested
    /// scope.
    pub error: Option<String>,
    pub error_description: Option<String>,
}

#[derive(Deserialize)]
struct FederatedTokenClaims {
    pub sub: String,
    /// Returned when `email` is in scope; without it the user is created
    /// without an email address.
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    /// Returned when `profile` is in scope.
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
}

async fn decode_using_jwk(
    token: &str,
    jwk_url: &str,
) -> Result<FederatedTokenClaims, OperationOutcomeError> {
    let header = jsonwebtoken::decode_header(token).map_err(|_| {
        OperationOutcomeError::error(
            IssueType::invalid(),
            "Failed to decode token header".to_string(),
        )
    })?;

    let res = reqwest::get(jwk_url).await.map_err(|_e| {
        OperationOutcomeError::error(
            IssueType::invalid(),
            "Failed to fetch JWKs from identity provider".to_string(),
        )
    })?;

    let jwk_set = res
        .json::<jsonwebtoken::jwk::JwkSet>()
        .await
        .map_err(|_e| {
            OperationOutcomeError::error(
                IssueType::invalid(),
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
            IssueType::invalid(),
            "No matching JWK found for token".to_string(),
        )
    })?;

    let decoding_key = DecodingKey::from_jwk(jwk).map_err(|_e| {
        OperationOutcomeError::error(
            IssueType::invalid(),
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
            IssueType::invalid(),
            "Failed to decode and verify token. Ensure openid is in scope and claims contain a sub claim.".to_string(),
        )
    })?;

    Ok(result.claims)
}

fn user_federated_id(idp: &IdentityProvider, sub: &str) -> Result<String, OperationOutcomeError> {
    let Some(id_prefix) = idp.id.as_ref() else {
        return Err(OperationOutcomeError::error(
            IssueType::invalid(),
            "Identity Provider is missing ID".to_string(),
        ));
    };

    let mut sha_hasher = Sha1::new();
    sha_hasher.update(sub.as_bytes());
    let hashed_user_sub_claim = URL_SAFE_NO_PAD.encode(sha_hasher.finalize());

    Ok(format!("{}|{}", id_prefix, hashed_user_sub_claim))
}

/// Access policies the project assigns to users signing in through `idp`, from
/// `Project.identityProviderSetting`.
///
/// A project can also assign policies with its own logic; that hook is not
/// implemented yet (see issue #939), and would run alongside these.
async fn default_access_policies<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    app_state: &Arc<ServerState<Repo, Search, Terminology>>,
    tenant: &TenantId,
    target_project: &ProjectId,
    idp: &IdentityProvider,
) -> Result<Vec<Reference>, OperationOutcomeError> {
    let Some(idp_id) = idp.id.as_ref() else {
        return Ok(vec![]);
    };
    let idp_reference = format!("{}/{}", ResourceType::IdentityProvider.as_ref(), idp_id);

    // Projects live in the tenant's system project.
    let project = app_state
        .fhir_client
        .read(
            Arc::new(ServerCTX::system(
                tenant.clone(),
                ProjectId::System,
                app_state.fhir_client.clone(),
                app_state.rate_limit.clone(),
            )),
            ResourceType::Project,
            target_project.as_ref().to_string(),
        )
        .await?;

    let Some(Resource::Project(project)) = project else {
        return Ok(vec![]);
    };

    Ok(project
        .identityProviderSetting
        .unwrap_or_default()
        .into_iter()
        .filter(|setting| {
            setting
                .identityProvider
                .reference
                .as_ref()
                .and_then(|r| r.value.as_ref())
                .is_some_and(|reference| reference == &idp_reference)
        })
        .flat_map(|setting| setting.defaultAccessPolicy.unwrap_or_default())
        .collect())
}

/// The user's name from the id token, when the provider returned one. Only the
/// `profile` scope gets these claims, so a federated user can legitimately have
/// no name.
fn federated_user_name(claims: &FederatedTokenClaims) -> Option<HumanName> {
    if claims.name.is_none() && claims.given_name.is_none() && claims.family_name.is_none() {
        return None;
    }

    Some(HumanName {
        text: claims.name.clone().map(|name| {
            Box::new(FHIRString {
                value: Some(name),
                ..Default::default()
            })
        }),
        given: claims.given_name.clone().map(|given| {
            vec![FHIRString {
                value: Some(given),
                ..Default::default()
            }]
        }),
        family: claims.family_name.clone().map(|family| {
            Box::new(FHIRString {
                value: Some(family),
                ..Default::default()
            })
        }),
        ..Default::default()
    })
}

async fn create_user_if_not_exists<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    app_state: &Arc<ServerState<Repo, Search, Terminology>>,
    tenant: &TenantId,
    target_project: &ProjectId,
    idp: &IdentityProvider,
    claims: &FederatedTokenClaims,
) -> Result<haste_fhir_model::r4::generated::resources::User, OperationOutcomeError> {
    let user_id = user_federated_id(idp, &claims.sub)?;

    let existing_user = app_state
        .fhir_client
        .batch(
            Arc::new(ServerCTX::system(
                tenant.clone(),
                target_project.clone(),
                app_state.fhir_client.clone(),
                app_state.rate_limit.clone(),
            )),
            Bundle {
                type_: BundleType::batch(),
                entry: Some(vec![
                    BundleEntry {
                        request: Some(BundleEntryRequest {
                            method: HttpVerb::get(),
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
                            method: HttpVerb::get(),
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

    // Batch responses come back in request order: the User read, then the
    // Membership search.
    let mut entries = existing_user.entry.unwrap_or_default().into_iter();

    let user_resource = entries
        .next()
        .and_then(|entry| entry.resource)
        .and_then(|resource| match *resource {
            Resource::User(user) => Some(user),
            _ => None,
        });

    let has_membership = entries
        .next()
        .and_then(|entry| entry.resource)
        .and_then(|resource| match *resource {
            Resource::Bundle(bundle) => bundle
                .entry
                .and_then(|entries| entries.into_iter().next())
                .and_then(|entry| entry.resource),
            _ => None,
        })
        .is_some_and(|resource| matches!(*resource, Resource::Membership(_)));

    if let Some(user) = user_resource
        && has_membership
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
                        transaction.rate_limit.clone(),
                    )),
                    ResourceType::User,
                    user_id.clone(),
                    Resource::User(User {
                        id: Some(user_id.clone()),
                        email: claims.email.clone().map(|email| {
                            Box::new(FHIRString {
                                value: Some(email),
                                ..Default::default()
                            })
                        }),
                        emailVerified: claims.email_verified.map(|verified| {
                            Box::new(FHIRBoolean {
                                value: Some(verified),
                                ..Default::default()
                            })
                        }),
                        name: federated_user_name(claims).map(Box::new),
                        role: UserRole::member(),
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
                        transaction.rate_limit.clone(),
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

            // Policies the project assigns to users of this identity provider.
            // Only done here, on first sign-in, so assignments an administrator
            // later removes are not recreated on the next sign-in.
            for policy in default_access_policies(app_state, tenant, target_project, idp).await? {
                transaction
                    .fhir_client
                    .create(
                        Arc::new(ServerCTX::system(
                            tenant.clone(),
                            target_project.clone(),
                            transaction.fhir_client.clone(),
                            transaction.rate_limit.clone(),
                        )),
                        ResourceType::AccessPolicyV2Assignment,
                        Resource::AccessPolicyV2Assignment(AccessPolicyV2Assignment {
                            accessPolicy: Box::new(policy),
                            link: Box::new(Reference {
                                reference: Some(Box::new(FHIRString {
                                    value: Some(format!(
                                        "{}/{}",
                                        ResourceType::Membership.as_ref(),
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
            }

            user
        };

        transaction.commit().await?;

        Ok(match created_user {
            Resource::User(user) => user,
            _ => {
                return Err(OperationOutcomeError::error(
                    IssueType::exception(),
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
    Query(CallbackQueryParams {
        code,
        state,
        error,
        error_description,
    }): Query<CallbackQueryParams>,
    State(app_state): State<Arc<ServerState<Repo, Search, Terminology>>>,
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    Cached(ProjectIdentifier { project }): Cached<ProjectIdentifier>,
    Cached(session): Cached<Session>,
) -> Result<Redirect, OperationOutcomeError> {
    let identity_provider = get_idp(
        &tenant,
        app_state.fhir_client.clone(),
        app_state.rate_limit.clone(),
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
                IssueType::invalid(),
                "Identity Provider is missing client ID".to_string(),
            )
        })?;

    let client_secret = identity_provider
        .oidc
        .as_ref()
        .and_then(|oidc| oidc.client.secret.as_ref())
        .and_then(|secret| secret.value.as_ref());

    // The provider sends `error` instead of `code` when it rejects the
    // authorization request, for instance when the client is not registered for
    // one of the scopes in `IdentityProvider.oidc.scopes`.
    if let Some(error) = error {
        tracing::error!(
            "Identity provider '{}' returned error '{}': {}",
            identity_provider_id,
            error,
            error_description.as_deref().unwrap_or("no description")
        );

        return Err(OperationOutcomeError::error(
            IssueType::invalid(),
            format!(
                "Identity provider returned an error: {}{}",
                error,
                error_description
                    .map(|description| format!(" ({})", description))
                    .unwrap_or_default()
            ),
        ));
    }

    if state.as_ref() != Some(&idp_session_info.state) {
        return Err(OperationOutcomeError::error(
            IssueType::invalid(),
            "State parameter does not match the stored session state.".to_string(),
        ));
    }

    let Some(code) = code else {
        return Err(OperationOutcomeError::error(
            IssueType::invalid(),
            "Identity provider callback is missing the authorization code.".to_string(),
        ));
    };

    if project != ProjectId::System && idp_session_info.project != project {
        return Err(OperationOutcomeError::error(
            IssueType::invalid(),
            "Project in session does not match the current project.".to_string(),
        ));
    }

    let federated_token_body = FederatedTokenBodyRequest {
        grant_type: GrantType::AuthorizationCode,
        code,
        redirect_uri: create_federated_callback_url(
            &app_state.config.api_uri,
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
                IssueType::invalid(),
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
                IssueType::invalid(),
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
                IssueType::invalid(),
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
            IssueType::invalid(),
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
                IssueType::invalid(),
                "Failed to parse token response from identity provider".to_string(),
            )
        })?;

    // A provider only returns an id token for an OIDC request, so this is what a
    // missing `openid` scope looks like: the user has authenticated, but there
    // is nothing to identify them by.
    let Some(id_token) = token_response_body.id_token else {
        return Err(OperationOutcomeError::error(
            IssueType::invalid(),
            "Identity provider did not return an id token. Check that its client is registered for the 'openid' scope.".to_string(),
        ));
    };

    let claims = decode_using_jwk(&id_token, jwk_url).await?;

    let user = create_user_if_not_exists(
        &app_state,
        &tenant,
        &idp_session_info.project,
        &identity_provider,
        &claims,
    )
    .await?;

    let Some(user_model) = TenantModelAdmin::<CreateUser, _, _, _, _>::read(
        app_state.repo.as_ref(),
        &tenant,
        &user.id.unwrap(),
    )
    .await?
    else {
        return Err(OperationOutcomeError::error(
            IssueType::exception(),
            "Failed to retrieve created federated user from repository".to_string(),
        ));
    };

    session::user::set_initial_authorization_state(app_state.repo.as_ref(), &session, user_model)
        .await?;

    let redirect_target = match session::user::get_authorization_state(&session).await? {
        Some(SessionAuthorizationState::MFARequired { .. }) => {
            totp_verification_route(&tenant, &idp_session_info.redirect_to)
        }
        _ => idp_session_info.redirect_to,
    };

    Ok(Redirect::to(&redirect_target))
}

pub fn create_federated_callback_url(
    api_url_string: &str,
    tenant: &TenantId,
    idp_id: &str,
) -> Result<String, OperationOutcomeError> {
    let Ok(api_url) = Url::parse(api_url_string) else {
        return Err(OperationOutcomeError::error(
            IssueType::exception(),
            "Invalid API_URL format".to_string(),
        ));
    };

    Ok(api_url
        .join(&format!(
            "w/{}/system/api/v1/oidc/federated/{}/callback",
            tenant.as_ref(),
            idp_id
        ))
        .unwrap()
        .to_string())
}
