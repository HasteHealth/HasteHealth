//! The keys a client registered for `private_key_jwt`: a JSON Web Key Set on
//! the `ClientApplication` (`jwks`), or one fetched from its `jwksUri` and
//! cached.

use crate::safe_fetch;
use haste_fhir_model::r4::generated::resources::ClientApplication;
use jsonwebtoken::jwk::{Jwk, JwkSet};
use moka::future::Cache;
use std::{
    sync::{Arc, LazyLock},
    time::Duration,
};

/// The largest JWK Set document accepted. A key is about a kilobyte.
pub const MAX_JWKS_BYTES: usize = 256 * 1024;
/// How long a fetched JWK Set is reused before it is fetched again anyway.
const CACHE_TTL: Duration = Duration::from_hours(1);
/// A `jwksUri` is fetched again when an assertion names a `kid` its cached
/// set lacks (a rotated key), or when no set could be fetched, but no more
/// often than this, so requests cannot make the server hammer the URL.
const REFRESH_INTERVAL: Duration = Duration::from_mins(1);

/// Why no key could be found for an assertion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyError {
    /// The client has neither `jwks` nor `jwksUri`.
    NotRegistered,
    /// The document is not a JWK Set with a `kid` on every key.
    InvalidJwks,
    /// `jwksUri` could not be fetched, or was refused.
    Fetch,
    /// No key in the set carries the `kid`.
    UnknownKid,
}

/// The JWK Sets fetched from `jwksUri`s.
struct RemoteKeys {
    /// The set last fetched from each URL, reused for `CACHE_TTL`.
    sets: Cache<String, Arc<JwkSet>>,
    /// What each URL's fetch in the last `REFRESH_INTERVAL` gave, a failure
    /// included. While an outcome is here its URL is not fetched again.
    recent: Cache<String, Result<Arc<JwkSet>, KeyError>>,
}

static REMOTE: LazyLock<RemoteKeys> = LazyLock::new(RemoteKeys::new);

impl RemoteKeys {
    fn new() -> Self {
        Self {
            sets: Cache::builder()
                .max_capacity(1_000)
                .time_to_live(CACHE_TTL)
                .build(),
            recent: Cache::builder()
                .max_capacity(1_000)
                .time_to_live(REFRESH_INTERVAL)
                .build(),
        }
    }

    /// The key `kid` of the set at `uri`. `fetch` runs when no cached set has
    /// the key (none was fetched yet, or the key was rotated in) and the URL
    /// was not fetched in the last `REFRESH_INTERVAL`. Concurrent calls for
    /// one URL share a single fetch, and a fetch that fails leaves the set
    /// cached before it in place.
    async fn find_key(
        &self,
        uri: &str,
        kid: &str,
        fetch: impl Future<Output = Result<JwkSet, KeyError>> + Send,
    ) -> Result<Jwk, KeyError> {
        if let Some(keys) = self.sets.get(uri).await
            && let Some(key) = keys.find(kid)
        {
            return Ok(key.clone());
        }

        let fetch_and_keep = async {
            let keys = Arc::new(fetch.await?);
            self.sets.insert(uri.to_string(), keys.clone()).await;
            Ok(keys)
        };
        let keys = self
            .recent
            .entry_by_ref(uri)
            .or_insert_with(fetch_and_keep)
            .await
            .into_value()?;

        keys.find(kid).cloned().ok_or(KeyError::UnknownKid)
    }
}

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

    REMOTE.find_key(uri, kid, fetch(uri)).await
}

/// Fetches and parses the JWK Set at `uri`.
async fn fetch(uri: &str) -> Result<JwkSet, KeyError> {
    let body = safe_fetch::fetch_public_https(uri, MAX_JWKS_BYTES)
        .await
        .map_err(|error| {
            tracing::warn!(%error, uri, "Could not fetch a client's jwksUri");
            KeyError::Fetch
        })?;

    parse_jwks(&body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    const URI: &str = "https://client.example.com/jwks.json";

    fn set(kid: &str) -> JwkSet {
        let document =
            format!(r#"{{"keys":[{{"kty":"RSA","kid":"{kid}","n":"AQAB","e":"AQAB"}}]}}"#);
        parse_jwks(document.as_bytes()).unwrap()
    }

    /// A fetch that counts itself and gives `outcome`.
    async fn counted(
        fetches: &AtomicUsize,
        outcome: Result<JwkSet, KeyError>,
    ) -> Result<JwkSet, KeyError> {
        fetches.fetch_add(1, Ordering::SeqCst);
        tokio::task::yield_now().await;
        outcome
    }

    /// What `REFRESH_INTERVAL` passing does to `uri`.
    async fn expire_recent(remote: &RemoteKeys) {
        remote.recent.invalidate(URI).await;
    }

    #[tokio::test]
    async fn a_cached_key_is_found_without_a_fetch() {
        let remote = RemoteKeys::new();
        let fetches = AtomicUsize::new(0);

        for _ in 0..3 {
            let key = remote
                .find_key(URI, "k1", counted(&fetches, Ok(set("k1"))))
                .await;
            assert!(key.is_ok());
            expire_recent(&remote).await;
        }

        assert_eq!(fetches.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn an_unknown_kid_fetches_again_once_per_interval() {
        let remote = RemoteKeys::new();
        let fetches = AtomicUsize::new(0);
        remote
            .find_key(URI, "k1", counted(&fetches, Ok(set("k1"))))
            .await
            .unwrap();

        // Within the interval the set just fetched answers.
        let key = remote
            .find_key(URI, "k2", counted(&fetches, Ok(set("k2"))))
            .await;
        assert_eq!(key.unwrap_err(), KeyError::UnknownKid);
        assert_eq!(fetches.load(Ordering::SeqCst), 1);

        // After it, the rotated key is found.
        expire_recent(&remote).await;
        let key = remote
            .find_key(URI, "k2", counted(&fetches, Ok(set("k2"))))
            .await;
        assert!(key.is_ok());
        assert_eq!(fetches.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn a_failed_fetch_is_not_repeated_within_the_interval() {
        let remote = RemoteKeys::new();
        let fetches = AtomicUsize::new(0);

        for failure in [KeyError::Fetch, KeyError::InvalidJwks] {
            for _ in 0..3 {
                let key = remote
                    .find_key(URI, "k1", counted(&fetches, Err(failure)))
                    .await;
                assert_eq!(key.unwrap_err(), failure);
            }
            expire_recent(&remote).await;
        }
        assert_eq!(fetches.load(Ordering::SeqCst), 2);

        let key = remote
            .find_key(URI, "k1", counted(&fetches, Ok(set("k1"))))
            .await;
        assert!(key.is_ok());
        assert_eq!(fetches.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn a_failed_fetch_keeps_the_set_cached_before_it() {
        let remote = RemoteKeys::new();
        let fetches = AtomicUsize::new(0);
        remote
            .find_key(URI, "k1", counted(&fetches, Ok(set("k1"))))
            .await
            .unwrap();
        expire_recent(&remote).await;

        let key = remote
            .find_key(URI, "k2", counted(&fetches, Err(KeyError::Fetch)))
            .await;
        assert_eq!(key.unwrap_err(), KeyError::Fetch);

        let key = remote
            .find_key(URI, "k1", counted(&fetches, Err(KeyError::Fetch)))
            .await;
        assert!(key.is_ok());
        assert_eq!(fetches.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn concurrent_lookups_share_one_fetch() {
        let remote = RemoteKeys::new();
        let fetches = AtomicUsize::new(0);
        let lookup = || remote.find_key(URI, "k1", counted(&fetches, Ok(set("k1"))));

        let (first, second, third) = tokio::join!(lookup(), lookup(), lookup());

        assert!(first.is_ok() && second.is_ok() && third.is_ok());
        assert_eq!(fetches.load(Ordering::SeqCst), 1);
    }
}
