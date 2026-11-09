//! The keys a client registered for `private_key_jwt`: a JSON Web Key Set on
//! the `ClientApplication` (`jwks`), or one fetched from its `jwksUri` and
//! cached.

use crate::safe_fetch::{self, FetchError};
use haste_fhir_model::r4::generated::resources::ClientApplication;
use jsonwebtoken::jwk::{Jwk, JwkSet};
use moka::future::Cache;
use std::{
    sync::{Arc, LazyLock},
    time::{Duration, Instant},
};

/// The largest JWK Set document accepted. A key is about a kilobyte.
pub const MAX_JWKS_BYTES: usize = 256 * 1024;
/// How long a fetched JWK Set is reused before it is fetched again anyway.
const CACHE_TTL: Duration = Duration::from_hours(1);
/// An assertion naming a `kid` the cached set lacks (a rotated key) fetches
/// the set again, but no more often than this, so unknown kids cannot make
/// the server hammer the URL.
const REFRESH_INTERVAL: Duration = Duration::from_mins(1);

/// Why no key could be found for an assertion.
#[derive(Debug)]
pub enum KeyError {
    /// The client has neither `jwks` nor `jwksUri`.
    NotRegistered,
    /// The document is not a JWK Set with a `kid` on every key.
    InvalidJwks,
    /// `jwksUri` could not be fetched, or was refused.
    Fetch(FetchError),
    /// No key in the set carries the `kid`.
    UnknownKid,
}

struct FetchedJwks {
    keys: JwkSet,
    fetched_at: Instant,
}

static REMOTE: LazyLock<Cache<String, Arc<FetchedJwks>>> = LazyLock::new(|| {
    Cache::builder()
        .max_capacity(1_000)
        .time_to_live(CACHE_TTL)
        .build()
});

/// Parses a JWK Set document. SMART App Launch requires every key to carry a
/// `kid`, which is how an assertion names the key that signed it.
pub fn parse_jwks(document: &[u8]) -> Result<JwkSet, KeyError> {
    let set: JwkSet = serde_json::from_slice(document).map_err(|_| KeyError::InvalidJwks)?;

    let every_key_has_kid = set.keys.iter().all(|key| {
        key.common
            .key_id
            .as_ref()
            .is_some_and(|kid| !kid.is_empty())
    });

    if set.keys.is_empty() || !every_key_has_kid {
        return Err(KeyError::InvalidJwks);
    }

    Ok(set)
}

fn value(string: Option<&str>) -> Option<&str> {
    string.map(str::trim).filter(|value| !value.is_empty())
}

/// The key `kid` of the keys `client` registered. `jwks` wins over `jwksUri`
/// when both are set, so a client that moves its keys inline is never
/// verified against a stale remote set.
pub async fn find_key(client: &ClientApplication, kid: &str) -> Result<Jwk, KeyError> {
    if let Some(document) = value(client.jwks.as_ref().and_then(|jwks| jwks.value.as_deref())) {
        return parse_jwks(document.as_bytes())?
            .find(kid)
            .cloned()
            .ok_or(KeyError::UnknownKid);
    }

    let Some(uri) = value(client.jwksUri.as_ref().and_then(|uri| uri.value.as_deref())) else {
        return Err(KeyError::NotRegistered);
    };

    find_remote_key(uri, kid).await
}

async fn find_remote_key(uri: &str, kid: &str) -> Result<Jwk, KeyError> {
    let cached = REMOTE.get(uri).await;

    if let Some(set) = &cached
        && let Some(key) = set.keys.find(kid)
    {
        return Ok(key.clone());
    }

    if cached.is_some_and(|set| set.fetched_at.elapsed() < REFRESH_INTERVAL) {
        return Err(KeyError::UnknownKid);
    }

    let body = safe_fetch::fetch_public_https(uri, MAX_JWKS_BYTES)
        .await
        .map_err(KeyError::Fetch)?;
    let fetched = Arc::new(FetchedJwks {
        keys: parse_jwks(&body)?,
        fetched_at: Instant::now(),
    });
    REMOTE.insert(uri.to_string(), fetched.clone()).await;

    fetched.keys.find(kid).cloned().ok_or(KeyError::UnknownKid)
}
