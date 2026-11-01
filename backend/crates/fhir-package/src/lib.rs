//! FHIR NPM packages, the form implementation guides are published in:
//! reading a package tarball ([`read`]) and fetching one from a package
//! registry such as packages.fhir.org ([`resolve_version`], [`download`]).
//!
//! A package is a gzipped tarball with a `package/` folder holding
//! `package.json`, the conformance resources as one JSON file each, and the
//! examples in `package/example/`. See
//! <https://hl7.org/fhir/packages.html>.

mod error;
mod registry;
mod tarball;

pub use error::PackageError;
pub use registry::{download, resolve_version};
pub use tarball::{Limits, pack, read};

use serde::Deserialize;
use std::{collections::BTreeMap, fmt::Display, str::FromStr};

/// A package's `package.json`, as far as installing it needs.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    /// The canonical URL its resources are published under.
    pub canonical: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "fhirVersions")]
    pub fhir_versions: Vec<String>,
    /// Package name to the version this one was built against.
    pub dependencies: BTreeMap<String, String>,
    pub directories: Directories,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Directories {
    /// The examples' folder, relative to the package folder. `example` when
    /// unset.
    pub example: Option<String>,
}

/// A JSON file in a package.
#[derive(Debug, Clone)]
pub struct PackageFile {
    /// Its path relative to the package folder.
    pub path: String,
    pub content: Vec<u8>,
}

/// A package read from its tarball.
#[derive(Debug, Clone)]
pub struct Package {
    pub manifest: Manifest,
    /// The JSON files directly in the package folder, other than
    /// `package.json` and the index: its conformance resources.
    pub resources: Vec<PackageFile>,
    /// The JSON files in its example folder.
    pub examples: Vec<PackageFile>,
    /// What the tarball expanded to, headers included.
    pub unpacked_bytes: u64,
}

/// A package to install, named the way registries and `package.json`
/// dependencies name one: `name@version`, `name#version` or `name` alone for
/// the latest version. A version may end in `.x` to take the latest match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageSpec {
    pub name: String,
    pub version: Option<String>,
}

impl FromStr for PackageSpec {
    type Err = PackageError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.trim();
        let (name, version) = match value.find(['@', '#']) {
            Some(at) => (&value[..at], Some(&value[at + 1..])),
            None => (value, None),
        };
        PackageSpec::new(name, version)
    }
}

impl PackageSpec {
    /// A spec for `name` at `version`, both checked.
    ///
    /// # Errors
    ///
    /// When the name or version has characters packages do not use.
    pub fn new(name: &str, version: Option<&str>) -> Result<Self, PackageError> {
        if !valid_name(name) {
            return Err(PackageError::InvalidName(name.to_string()));
        }
        let version = match version.map(str::trim) {
            None | Some("") => None,
            Some(version) if valid_version(version) => Some(version.to_string()),
            Some(version) => return Err(PackageError::InvalidVersion(version.to_string())),
        };
        Ok(PackageSpec {
            name: name.to_string(),
            version,
        })
    }
}

impl Display for PackageSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.version {
            Some(version) => write!(f, "{}@{version}", self.name),
            None => write!(f, "{}", self.name),
        }
    }
}

/// Package names are lowercase in practice (`hl7.fhir.us.core`); any case is
/// accepted here. They become a URL path segment, so nothing else is.
#[must_use]
pub fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 214
        && name.starts_with(|c: char| c.is_ascii_alphanumeric())
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
}

/// A semver-like version (`6.1.0`, `2.0.0-ballot`, `6.1.x`). It becomes a URL
/// path segment, so nothing else is accepted.
#[must_use]
pub fn valid_version(version: &str) -> bool {
    !version.is_empty()
        && version.len() <= 64
        && version.starts_with(|c: char| c.is_ascii_alphanumeric())
        && version
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '+'))
}
