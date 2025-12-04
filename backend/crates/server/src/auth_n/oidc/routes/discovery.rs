use crate::{
    auth_n::oidc::{
        error::{OIDCError, OIDCErrorCode},
        routes::{AUTH_NESTED_PATH, authorize, jwks, token},
    },
    extract::path_tenant::{ProjectIdentifier, TenantIdentifier},
    services::AppState,
};
use axum::{
    extract::{FromRequestParts, Json, Path, State},
    http::request::Parts,
    response::{IntoResponse, Response},
};
use axum_extra::extract::Cached;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, TenantId, scopes::Scopes};
use haste_repository::Repository;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use url::Url;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WellKnownDiscoveryDocument {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub jwks_uri: String,
    pub token_endpoint: String,
    pub scopes_supported: Vec<String>,
    pub response_types_supported: Vec<String>,
    pub token_endpoint_auth_methods_supported: Vec<String>,
    pub id_token_signing_alg_values_supported: Vec<String>,
    pub subject_types_supported: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OAuthProtectedResourceDocument {
    /**
     * REQUIRED.  The protected resource's resource identifier, as
     * defined in Section 1.2.
     */
    resource: String,

    /**
     * OPTIONAL.  JSON array containing a list of OAuth authorization
     * server issuer identifiers, as defined in [RFC8414], for
     * authorization servers that can be used with this protected
     * resource.  Protected resources MAY choose not to advertise some
     * supported authorization servers even when this parameter is used.
     * In some use cases, the set of authorization servers will not be
     * enumerable, in which case this metadata parameter would not be
     * used.
     */
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    authorization_servers: Option<Vec<String>>,

    /**
     * OPTIONAL.  URL of the protected resource's JSON Web Key (JWK) Set
     * [JWK] document.  This contains public keys belonging to the
     * protected resource, such as signing key(s) that the resource
     * server uses to sign resource responses.  This URL MUST use the
     * https scheme.  When both signing and encryption keys are made
     * available, a use (public key use) parameter value is REQUIRED for
     * all keys in the referenced JWK Set to indicate each key's intended
     * usage.
     */
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    jwks_uri: Option<String>,

    /**
     * RECOMMENDED.  JSON array containing a list of scope values, as
     * defined in OAuth 2.0 [RFC6749], that are used in authorization
     * requests to request access to this protected resource.  Protected
     * resources MAY choose not to advertise some scope values supported
     * even when this parameter is used.
     */
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    scopes_supported: Option<Vec<String>>,

    /**
     * OPTIONAL.  JSON array containing a list of the supported methods
     * of sending an OAuth 2.0 bearer token [RFC6750] to the protected
     * resource.  Defined values are ["header", "body", "query"],
     * corresponding to Sections 2.1, 2.2, and 2.3 of [RFC6750].  The
     * empty array [] can be used to indicate that no bearer methods are
     * supported.  If this entry is omitted, no default bearer methods
     * supported are implied, nor does its absence indicate that they are
     * not supported.
     */
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    bearer_methods_supported: Option<Vec<String>>,

    /**
     * OPTIONAL.  JSON array containing a list of the JWS [JWS] signing
     * algorithms (alg values) [JWA] supported by the protected resource
     * for signing resource responses, for instance, as described in
     * [FAPI.MessageSigning].  No default algorithms are implied if this
     * entry is omitted.  The value none MUST NOT be used.
     */
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    resource_signing_alg_values_supported: Option<Vec<String>>,

    /**
     * Human-readable name of the protected resource intended for display
     * to the end user.  It is RECOMMENDED that protected resource
     * metadata include this field.  The value of this field MAY be
     * internationalized, as described in Section 2.1.
     */
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    resource_name: Option<String>,

    /**
     * OPTIONAL.  URL of a page containing human-readable information
     * that developers might want or need to know when using the
     * protected resource.  The value of this field MAY be
     * internationalized, as described in Section 2.1.
     */
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    resource_documentation: Option<String>,

    /**
     * OPTIONAL.  URL of a page containing human-readable information
     * about the protected resource's requirements on how the client can
     * use the data provided by the protected resource.  The value of
     * this field MAY be internationalized, as described in Section 2.1.
     */
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    resource_policy_uri: Option<String>,

    /**
     * OPTIONAL.  URL of a page containing human-readable information
     * about the protected resource's terms of service.  The value of
     * this field MAY be internationalized, as described in Section 2.1.
     */
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    resource_tos_uri: Option<String>,

    /**
     * OPTIONAL.  Boolean value indicating protected resource support for
     * mutual-TLS client certificate-bound access tokens [RFC8705].  If
     * omitted, the default value is false.
     */
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    tls_client_certificate_bound_access_tokens: Option<bool>,

    /**
     * OPTIONAL.  JSON array containing a list of the authorization
     * details type values supported by the resource server when the
     * authorization_details request parameter [RFC9396] is used.
     */
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    authorization_details_types_supported: Option<Vec<String>>,

    /**
     * OPTIONAL.  JSON array containing a list of the JWS alg values
     * (from the "JSON Web Signature and Encryption Algorithms" registry
     * [IANA.JOSE]) supported by the resource server for validating
     * Demonstrating Proof of Possession (DPoP) proof JWTs [RFC9449].
     */
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    dpop_signing_alg_values_supported: Option<Vec<String>>,

    /**
     * OPTIONAL.  Boolean value specifying whether the protected resource
     * always requires the use of DPoP-bound access tokens [RFC9449].  If
     * omitted, the default value is false.
     */
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    dpop_bound_access_tokens_required: Option<bool>,
}

fn construct_oidc_route(tenant: &TenantId, project: &ProjectId, path: &str) -> String {
    format!(
        "/w/{}/{}/api/v1/oidc{}",
        tenant.as_ref(),
        project.as_ref(),
        path
    )
}

#[derive(Deserialize, Clone)]
pub struct ResourcePath {
    pub resource: String,
}

impl<S: Send + Sync> FromRequestParts<S> for ResourcePath {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(resource) = Path::<ResourcePath>::from_request_parts(parts, state)
            .await
            .map_err(|err| err.into_response())?;

        Ok(resource)
    }
}

pub async fn oauth_protected_resource<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    Cached(ResourcePath { resource }): Cached<ResourcePath>,
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    Cached(ProjectIdentifier { project }): Cached<ProjectIdentifier>,
    State(state): State<Arc<AppState<Repo, Search, Terminology>>>,
) -> Result<Json<OAuthProtectedResourceDocument>, OIDCError> {
    let api_url_string = state
        .config
        .get(crate::ServerEnvironmentVariables::APIURI)
        .unwrap_or_default();

    if api_url_string.is_empty() {
        return Err(OIDCError::new(
            OIDCErrorCode::ServerError,
            Some("API_URL is not set in the configuration".to_string()),
            None,
        ));
    }

    let Ok(api_url) = Url::parse(&api_url_string) else {
        return Err(OIDCError::new(
            OIDCErrorCode::ServerError,
            Some("Invalid API_URL format".to_string()),
            None,
        ));
    };

    // Default to openid profile user/*.* scopes for FHIR access.
    let default_scopes = Scopes::try_from("openid profile user/*.*").unwrap_or_default();

    let oauth_protected_resource = OAuthProtectedResourceDocument {
        resource: api_url
            .join(&format!(
                "/w/{}/{}/{}",
                tenant.as_ref(),
                project.as_ref(),
                resource
            ))
            .unwrap()
            .to_string(),

        authorization_servers: Some(vec![
            api_url
                .join(&format!("/w/{}/{}", tenant.as_ref(), project.as_ref()))
                .unwrap()
                .to_string(),
        ]),
        jwks_uri: None,

        scopes_supported: Some(
            default_scopes
                .0
                .into_iter()
                .map(|s| String::from(s))
                .collect::<Vec<_>>(),
        ),
        bearer_methods_supported: None,
        resource_signing_alg_values_supported: None,
        resource_name: None,
        resource_documentation: None,
        resource_policy_uri: None,
        resource_tos_uri: None,
        tls_client_certificate_bound_access_tokens: None,
        authorization_details_types_supported: None,
        dpop_signing_alg_values_supported: None,
        dpop_bound_access_tokens_required: None,
    };

    Ok(Json(oauth_protected_resource))
}

pub async fn openid_configuration<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    Cached(ProjectIdentifier { project }): Cached<ProjectIdentifier>,
    State(state): State<Arc<AppState<Repo, Search, Terminology>>>,
) -> Result<Json<WellKnownDiscoveryDocument>, OIDCError> {
    let api_url_string = state
        .config
        .get(crate::ServerEnvironmentVariables::APIURI)
        .unwrap_or_default();

    if api_url_string.is_empty() {
        return Err(OIDCError::new(
            OIDCErrorCode::ServerError,
            Some("API_URL is not set in the configuration".to_string()),
            None,
        ));
    }

    let Ok(api_url) = Url::parse(&api_url_string) else {
        return Err(OIDCError::new(
            OIDCErrorCode::ServerError,
            Some("Invalid API_URL format".to_string()),
            None,
        ));
    };

    let authorize_path = construct_oidc_route(
        &tenant,
        &project,
        &(AUTH_NESTED_PATH.to_string() + authorize::AuthorizePath.to_string().as_str()),
    );

    let token_path = construct_oidc_route(
        &tenant,
        &project,
        &(AUTH_NESTED_PATH.to_string() + token::TokenPath.to_string().as_str()),
    );

    let jwks_path = construct_oidc_route(
        &tenant,
        &project,
        &(AUTH_NESTED_PATH.to_string() + jwks::JWKSPath.to_string().as_str()),
    );

    let oidc_response = WellKnownDiscoveryDocument {
        issuer: api_url.to_string(),
        authorization_endpoint: api_url.join(&authorize_path).unwrap().to_string(),
        token_endpoint: api_url.join(&token_path).unwrap().to_string(),
        jwks_uri: api_url.join(&jwks_path).unwrap().to_string(),
        scopes_supported: vec![
            "openid".to_string(),
            "profile".to_string(),
            "email".to_string(),
            "offline_access".to_string(),
        ],
        response_types_supported: vec![
            "code".to_string(),
            "id_token".to_string(),
            "id_token token".to_string(),
        ],
        token_endpoint_auth_methods_supported: vec![
            "client_secret_basic".to_string(),
            "client_secret_post".to_string(),
        ],
        id_token_signing_alg_values_supported: vec!["RS256".to_string()],
        subject_types_supported: vec!["public".to_string()],
    };

    Ok(Json(oidc_response))
}
