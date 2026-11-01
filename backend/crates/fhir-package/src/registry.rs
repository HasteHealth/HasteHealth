//! A FHIR package registry's npm-style API, as packages.fhir.org serves it:
//! `GET {registry}/{name}` lists a package's versions and
//! `GET {registry}/{name}/{version}` is the version's tarball.

use crate::{PackageError, valid_name, valid_version};
use serde::Deserialize;
use std::{collections::BTreeMap, sync::LazyLock, time::Duration};

static HTTP: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_mins(2))
        .user_agent(concat!("haste-health/", env!("CARGO_PKG_VERSION")))
        .build()
        .expect("failed to build the package registry HTTP client")
});

/// What `GET {registry}/{name}` answers, as far as picking a version needs.
#[derive(Deserialize, Default)]
#[serde(default)]
struct PackageVersions {
    #[serde(rename = "dist-tags")]
    dist_tags: BTreeMap<String, String>,
    versions: BTreeMap<String, serde_json::Value>,
}

fn url(registry: &str, segments: &[&str]) -> String {
    let mut url = registry.trim_end_matches('/').to_string();
    for segment in segments {
        url.push('/');
        url.push_str(segment);
    }
    url
}

fn registry_error(error: &reqwest::Error) -> PackageError {
    PackageError::Registry(error.to_string())
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
/// When the name or version is malformed, the registry does not know the
/// package or has no matching version, or cannot be reached.
pub async fn resolve_version(
    registry: &str,
    name: &str,
    requested: Option<&str>,
) -> Result<String, PackageError> {
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
        .get(url(registry, &[name]))
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| registry_error(&e))?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(PackageError::NotFound(name.to_string()));
    }
    let body = response
        .error_for_status()
        .map_err(|e| registry_error(&e))?
        .bytes()
        .await
        .map_err(|e| registry_error(&e))?;
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
/// When the name or version is malformed, the registry does not have that
/// version, the download is too large, or the registry cannot be reached.
pub async fn download(
    registry: &str,
    name: &str,
    version: &str,
    max_bytes: u64,
) -> Result<Vec<u8>, PackageError> {
    if !valid_name(name) {
        return Err(PackageError::InvalidName(name.to_string()));
    }
    if !valid_version(version) {
        return Err(PackageError::InvalidVersion(version.to_string()));
    }

    let mut response = HTTP
        .get(url(registry, &[name, version]))
        .send()
        .await
        .map_err(|e| registry_error(&e))?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(PackageError::NotFound(format!("{name}@{version}")));
    }
    response = response
        .error_for_status()
        .map_err(|e| registry_error(&e))?;
    if response
        .content_length()
        .is_some_and(|length| length > max_bytes)
    {
        return Err(PackageError::TooLarge(max_bytes));
    }

    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|e| registry_error(&e))? {
        if (bytes.len() + chunk.len()) as u64 > max_bytes {
            return Err(PackageError::TooLarge(max_bytes));
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}
