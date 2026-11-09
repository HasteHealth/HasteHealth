//! `private_key_jwt` client authentication at the token endpoint (RFC 7523
//! section 2.2): the client sends a JWT signed with one of the keys it
//! registered, in place of a client secret. SMART App Launch requires it for
//! Backend Services and offers it to apps as `client-confidential-asymmetric`;
//! its rules for the assertion are the ones checked here
//! (<https://hl7.org/fhir/smart-app-launch/STU2.2/client-confidential-asymmetric.html>).

pub mod jwks;

use crate::{
    auth_n::oidc::{
        error::{OIDCError, OIDCErrorCode},
        extract::client_app::find_client_app,
        routes::discovery::token_endpoint_url,
        schemas::token_body::OAuth2TokenBody,
    },
    services::ServerState,
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use haste_fhir_model::r4::generated::resources::ClientApplication;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, TenantId};
use haste_repository::Repository;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, errors::ErrorKind};
use jwks::KeyError;
use serde::Deserialize;
use std::time::Duration;

/// The only `client_assertion_type` accepted (RFC 7523 section 2.2).
pub const JWT_BEARER_ASSERTION_TYPE: &str =
    "urn:ietf:params:oauth:client-assertion-type:jwt-bearer";
/// The signing algorithms accepted, as JWS `alg` names: the two SMART App
/// Launch requires a server to support.
pub const SIGNING_ALGORITHMS: [&str; 2] = ["RS384", "ES384"];
const ACCEPTED_ALGORITHMS: [Algorithm; 2] = [Algorithm::RS384, Algorithm::ES384];
/// An assertion's `exp` may be at most this far ahead (SMART App Launch).
const MAX_LIFETIME_SECONDS: u64 = 5 * 60;
/// The clock skew tolerated between the client and the server.
const LEEWAY_SECONDS: u64 = 60;

/// The claims read once the signature is verified. `iss`, `sub`, `aud` and
/// `exp` are checked by the validation; `jti` has to be present (a missing
/// one fails here) and not used before.
#[derive(Deserialize)]
struct AssertionClaims {
    exp: u64,
    jti: String,
}

/// Who the assertion says it is from. Read before the signature can be
/// verified, because the key to verify it with is the claimed client's; it
/// is trusted only once that verification has passed.
#[derive(Deserialize)]
struct ClaimedIdentity {
    iss: String,
    sub: String,
}

/// The identity `assertion` claims, from its payload as sent: nothing about
/// it is verified yet.
fn claimed_identity(assertion: &str) -> Option<ClaimedIdentity> {
    let payload = assertion.split('.').nth(1)?;
    let bytes = URL_SAFE_NO_PAD.decode(payload).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn invalid_client(description: impl Into<String>) -> OIDCError {
    OIDCError::new(OIDCErrorCode::InvalidClient, Some(description.into()), None)
}

fn invalid_request(description: impl Into<String>) -> OIDCError {
    OIDCError::new(
        OIDCErrorCode::InvalidRequest,
        Some(description.into()),
        None,
    )
}

/// Whether `body` authenticates its client with an assertion rather than a
/// secret. Either parameter alone counts, so a half-formed attempt is
/// answered as an assertion problem instead of being treated as a public
/// client.
#[must_use]
pub fn is_assertion_request(body: &OAuth2TokenBody) -> bool {
    body.client_assertion.is_some() || body.client_assertion_type.is_some()
}

/// Whether `client` registered keys, which makes it a confidential client
/// that authenticates with an assertion.
#[must_use]
pub fn has_registered_keys(client: &ClientApplication) -> bool {
    let set = |value: Option<&str>| value.is_some_and(|value| !value.trim().is_empty());

    set(client.jwks.as_ref().and_then(|jwks| jwks.value.as_deref()))
        || set(client.jwksUri.as_ref().and_then(|uri| uri.value.as_deref()))
}

/// What a verification failure tells the client. Reasons stay specific, since
/// nothing here is secret and a developer setting up a client needs them.
fn describe(error: &jsonwebtoken::errors::Error) -> String {
    match error.kind() {
        ErrorKind::ExpiredSignature => "The client assertion has expired.".to_string(),
        ErrorKind::ImmatureSignature => "The client assertion is not valid yet (nbf).".to_string(),
        ErrorKind::InvalidAudience => {
            "The client assertion's aud must be this token endpoint's URL.".to_string()
        }
        ErrorKind::InvalidIssuer | ErrorKind::InvalidSubject => {
            "The client assertion's iss and sub must both be the client_id.".to_string()
        }
        ErrorKind::InvalidSignature => {
            "The client assertion's signature does not verify against the key its kid names."
                .to_string()
        }
        ErrorKind::InvalidAlgorithm => {
            "The key the client assertion's kid names does not match its alg.".to_string()
        }
        ErrorKind::MissingRequiredClaim(claim) => {
            format!("The client assertion lacks the {claim} claim.")
        }
        ErrorKind::Json(_) => "The client assertion must carry exp and jti claims.".to_string(),
        _ => "The client assertion could not be verified.".to_string(),
    }
}

fn key_error(kid: &str, error: KeyError) -> OIDCError {
    match error {
        KeyError::NotRegistered => invalid_client(
            "The client has no public keys registered (ClientApplication.jwks or jwksUri).",
        ),
        KeyError::InvalidJwks => {
            invalid_client("The client's registered keys are not a JSON Web Key Set.")
        }
        KeyError::Fetch(error) => {
            tracing::warn!(%error, "Could not fetch a client's jwksUri");
            invalid_client("The client's jwksUri could not be fetched.")
        }
        KeyError::UnknownKid => invalid_client(format!(
            "No registered key has the kid '{kid}' the client assertion names."
        )),
    }
}

/// Authenticates the client a token request's `client_assertion` is from and
/// returns it.
///
/// The assertion must be signed with RS384 or ES384 by a key the client
/// registered, found by the header's `kid`; its `iss` and `sub` must be the
/// `client_id`, its `aud` this token endpoint, its `exp` at most five minutes
/// ahead, and its `jti` unused. A failed check answers `invalid_client`; a
/// malformed request `invalid_request`.
pub async fn authenticate<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    state: &ServerState<Repo, Search, Terminology>,
    tenant: &TenantId,
    project: &ProjectId,
    body: &OAuth2TokenBody,
) -> Result<ClientApplication, OIDCError> {
    if body.client_assertion_type.as_deref() != Some(JWT_BEARER_ASSERTION_TYPE) {
        return Err(invalid_request(format!(
            "client_assertion_type must be {JWT_BEARER_ASSERTION_TYPE}."
        )));
    }

    let Some(assertion) = body.client_assertion.as_deref() else {
        return Err(invalid_request(
            "client_assertion is required with client_assertion_type.",
        ));
    };

    // RFC 6749 section 2.3: one authentication method per request.
    if body.client_secret.is_some() {
        return Err(invalid_request(
            "A request authenticates with a client_secret or a client_assertion, not both.",
        ));
    }

    let header = jsonwebtoken::decode_header(assertion)
        .map_err(|_| invalid_client("The client assertion is not a JWT."))?;

    if !ACCEPTED_ALGORITHMS.contains(&header.alg) {
        return Err(invalid_client(
            "The client assertion must be signed with RS384 or ES384.",
        ));
    }

    let Some(kid) = header.kid.filter(|kid| !kid.is_empty()) else {
        return Err(invalid_client(
            "The client assertion's header must name its signing key in kid.",
        ));
    };

    // Unverified at this point: it only says whose key to verify with.
    let identity = claimed_identity(assertion)
        .ok_or_else(|| invalid_client("The client assertion must carry iss and sub claims."))?;

    if identity.iss != identity.sub {
        return Err(invalid_client(
            "The client assertion's iss and sub must both be the client_id.",
        ));
    }

    if body
        .client_id
        .as_ref()
        .is_some_and(|client_id| *client_id != identity.iss)
    {
        return Err(invalid_client(
            "client_id does not match the client assertion's iss.",
        ));
    }

    let client_id = identity.iss;
    let client = find_client_app(state, tenant.clone(), project.clone(), client_id.clone()).await?;

    let key = jwks::find_key(&client, &kid)
        .await
        .map_err(|error| key_error(&kid, error))?;
    let decoding_key = DecodingKey::from_jwk(&key).map_err(|_| {
        invalid_client(format!(
            "The registered key '{kid}' cannot be used to verify a signature."
        ))
    })?;

    let audience = token_endpoint_url(tenant, project, &state.config.api_uri)?;

    let mut validation = Validation::new(header.alg);
    validation.leeway = LEEWAY_SECONDS;
    validation.set_required_spec_claims(&["exp", "iss", "sub", "aud"]);
    validation.set_issuer(&[client_id.as_str()]);
    validation.sub = Some(client_id.clone());
    validation.set_audience(&[audience.as_str()]);

    let claims = jsonwebtoken::decode::<AssertionClaims>(assertion, &decoding_key, &validation)
        .map_err(|error| invalid_client(describe(&error)))?
        .claims;

    let now = jsonwebtoken::get_current_timestamp();
    if claims.exp > now + MAX_LIFETIME_SECONDS + LEEWAY_SECONDS {
        return Err(invalid_client(
            "The client assertion's exp may be at most five minutes ahead.",
        ));
    }

    // Remembered for as long as the assertion could still be presented.
    let lifetime = Duration::from_secs(claims.exp.saturating_sub(now) + LEEWAY_SECONDS);
    let unused = state
        .repo
        .record_client_assertion_jti(tenant, project, &client_id, &claims.jti, lifetime)
        .await
        .map_err(|error| {
            tracing::error!(%error, "Could not record a client assertion's jti");
            OIDCError::new(
                OIDCErrorCode::ServerError,
                Some("Failed to check the client assertion for replay.".to_string()),
                None,
            )
        })?;

    if !unused {
        return Err(invalid_client(
            "The client assertion's jti has already been used.",
        ));
    }

    Ok(client)
}
