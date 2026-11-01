//! A FHIR package registry's npm-style API, as packages.fhir.org serves it:
//! `GET {registry}/{name}` lists a package's versions and
//! `GET {registry}/{name}/{version}` is the version's tarball.
//!
//! Requests go only to the configured registry: callers name packages and
//! never give a URL, the registry must be https, and a redirect is followed
//! only while it stays on the registry's scheme, host and port.

use crate::{PackageError, valid_name, valid_version};
use reqwest::{Url, redirect};
use serde::Deserialize;
use std::{collections::BTreeMap, sync::LazyLock, time::Duration};

/// The most redirects followed within the registry for one request.
const MAX_REDIRECTS: usize = 5;

static HTTP: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_mins(2))
        .https_only(true)
        .redirect(redirect::Policy::custom(within_registry))
        .user_agent(concat!("haste-health/", env!("CARGO_PKG_VERSION")))
        .build()
        .expect("failed to build the package registry HTTP client")
});

/// Follows a redirect only to the origin the request started at.
fn within_registry(attempt: redirect::Attempt) -> redirect::Action {
    let start = attempt.previous().first().map(Url::origin);
    if start.is_none_or(|start| start != attempt.url().origin()) {
        attempt.error("the registry redirected to another host")
    } else if attempt.previous().len() > MAX_REDIRECTS {
        attempt.error("too many redirects")
    } else {
        attempt.follow()
    }
}

/// The configured registry as a base URL. An empty setting means the server
/// does not fetch packages; anything but an https URL with a host is refused.
fn registry_base(registry: &str) -> Result<Url, PackageError> {
    let registry = registry.trim();
    if registry.is_empty() {
        return Err(PackageError::RegistryDisabled);
    }
    let base = Url::parse(registry).map_err(|_| PackageError::InvalidRegistry)?;
    if base.scheme() != "https"
        || base.host().is_none()
        || base.query().is_some()
        || base.fragment().is_some()
    {
        return Err(PackageError::InvalidRegistry);
    }
    Ok(base)
}

/// What `GET {registry}/{name}` answers, as far as picking a version needs.
#[derive(Deserialize, Default)]
#[serde(default)]
struct PackageVersions {
    #[serde(rename = "dist-tags")]
    dist_tags: BTreeMap<String, String>,
    versions: BTreeMap<String, serde_json::Value>,
}

fn url(registry: &Url, segments: &[&str]) -> Url {
    let mut url = registry.clone();
    if let Ok(mut path) = url.path_segments_mut() {
        path.pop_if_empty().extend(segments);
    }
    url
}

/// What went wrong, with its causes but not the request URL: a registry URL
/// may carry credentials, and the message goes back to the caller.
fn registry_error(error: reqwest::Error) -> PackageError {
    let error = error.without_url();
    let mut message = error.to_string();
    let mut source = std::error::Error::source(&error);
    while let Some(cause) = source {
        message.push_str(": ");
        message.push_str(&cause.to_string());
        source = cause.source();
    }
    PackageError::Registry(message)
}

/// Whether `requested` names one version, so the registry need not be asked
/// which versions exist.
fn is_exact(requested: &str) -> bool {
    requested != "latest"
        && !requested
            .split(['.', '-', '+'])
            .any(|part| matches!(part, "x" | "X" | "*"))
}

/// The newest of `versions` that `requested` (`6.1.x`) matches. Releases
/// sort by semver; a version that is not semver only matches exactly.
fn newest_match<'a>(versions: impl Iterator<Item = &'a String>, requested: &str) -> Option<String> {
    let prefix = requested
        .split('.')
        .take_while(|part| !matches!(*part, "x" | "X" | "*"))
        .fold(String::new(), |mut prefix, part| {
            prefix.push_str(part);
            prefix.push('.');
            prefix
        });
    versions
        .filter(|version| version.starts_with(&prefix))
        .filter_map(|version| {
            semver::Version::parse(version)
                .ok()
                .filter(|parsed| parsed.pre.is_empty())
                .map(|parsed| (parsed, version))
        })
        .max_by(|(a, _), (b, _)| a.cmp(b))
        .map(|(_, version)| version.clone())
}

/// The version of `name` to install: `requested` when it names one exactly,
/// else the registry's latest (no version, or `latest`) or newest match
/// (`6.1.x`).
///
/// # Errors
///
/// When the server does not fetch packages, the name or version is malformed,
/// the registry does not know the package or has no matching version, or
/// cannot be reached.
pub async fn resolve_version(
    registry: &str,
    name: &str,
    requested: Option<&str>,
) -> Result<String, PackageError> {
    let registry = registry_base(registry)?;
    if !valid_name(name) {
        return Err(PackageError::InvalidName(name.to_string()));
    }
    if let Some(requested) = requested {
        if !valid_version(requested) {
            return Err(PackageError::InvalidVersion(requested.to_string()));
        }
        if is_exact(requested) {
            return Ok(requested.to_string());
        }
    }

    let response = HTTP
        .get(url(&registry, &[name]))
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(registry_error)?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(PackageError::NotFound(name.to_string()));
    }
    let body = response
        .error_for_status()
        .map_err(registry_error)?
        .bytes()
        .await
        .map_err(registry_error)?;
    let versions = serde_json::from_slice::<PackageVersions>(&body)
        .map_err(|e| PackageError::Registry(format!("unexpected response for '{name}': {e}")))?;

    let requested = requested.unwrap_or("latest");
    let resolved = if requested == "latest" {
        versions.dist_tags.get("latest").cloned()
    } else {
        newest_match(versions.versions.keys(), requested)
    };
    resolved.ok_or_else(|| PackageError::NoMatchingVersion(name.to_string(), requested.to_string()))
}

/// Downloads `name` at `version` from `registry`, refusing a tarball larger
/// than `max_bytes`.
///
/// # Errors
///
/// When the server does not fetch packages, the name or version is malformed,
/// the registry does not have that version, the download is too large, or the
/// registry cannot be reached.
pub async fn download(
    registry: &str,
    name: &str,
    version: &str,
    max_bytes: u64,
) -> Result<Vec<u8>, PackageError> {
    let registry = registry_base(registry)?;
    if !valid_name(name) {
        return Err(PackageError::InvalidName(name.to_string()));
    }
    if !valid_version(version) {
        return Err(PackageError::InvalidVersion(version.to_string()));
    }

    let mut response = HTTP
        .get(url(&registry, &[name, version]))
        .send()
        .await
        .map_err(registry_error)?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(PackageError::NotFound(format!("{name}@{version}")));
    }
    response = response.error_for_status().map_err(registry_error)?;
    if response
        .content_length()
        .is_some_and(|length| length > max_bytes)
    {
        return Err(PackageError::TooLarge(max_bytes));
    }

    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(registry_error)? {
        if (bytes.len() + chunk.len()) as u64 > max_bytes {
            return Err(PackageError::TooLarge(max_bytes));
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}
