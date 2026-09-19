//! Applies patches and rules to externally provided FHIR artifacts (for example
//! the HL7 core package) without editing the files that came from upstream.
//!
//! A package opts in with a `patches/manifest.toml`:
//!
//! ```toml
//! # Upstream files or directories, relative to this manifest. Every `*.json`
//! # resource found is patched and written next to it as `<name>.min.json`.
//! sources = ["../definitions/hl7"]
//!
//! # Rules are transforms implemented in Rust (see `rules`) that run over every
//! # resource, optionally limited to some resource types.
//! [[rules]]
//! name = "clippy-doc-markdown"
//! ```
//!
//! Every other `*.json` file under `patches/` is a [`PatchFile`]: a description
//! plus RFC 6902 JSON Patch operations scoped to a single resource, which is
//! addressed by `resourceType` and `id` or `url` rather than by its position in
//! a bundle.
//!
//! The pipeline for each resource is: targeted patches (in file path order),
//! then rules (in manifest order), then minimization. Every change made by a
//! patch or rule is recorded as a [`Change`], so the complete set of
//! differences from upstream can be listed without diffing the output files.

mod minimize;
pub mod rules;

use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::HashMap,
    fmt,
    path::{Component, Path, PathBuf},
};
use walkdir::WalkDir;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to read '{path}': {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("invalid manifest '{path}': {source}")]
    Manifest {
        path: PathBuf,
        source: Box<toml::de::Error>,
    },
    #[error("invalid JSON in '{path}': {source}")]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("unknown rule '{0}' (known rules: {known})", known = rules::names().join(", "))]
    UnknownRule(String),
    #[error("patch '{patch}' has a target with neither an 'id' nor a 'url'")]
    UnaddressedTarget { patch: PathBuf },
    #[error("patch '{patch}' target {target} matched {count} resources, expected exactly 1")]
    TargetMatchCount {
        patch: PathBuf,
        target: Target,
        count: usize,
    },
    #[error("patch '{patch}' failed on {resource}: {source}")]
    Apply {
        patch: PathBuf,
        resource: String,
        source: json_patch::PatchError,
    },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    sources: Vec<PathBuf>,
    #[serde(default)]
    rules: Vec<RuleConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RuleConfig {
    name: String,
    /// Restrict the rule to these resource types. Empty means every resource.
    #[serde(default)]
    resource_types: Vec<String>,
}

/// A file of targeted edits to upstream resources.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatchFile {
    /// Why the upstream resources are being changed.
    pub description: String,
    pub patches: Vec<ResourcePatch>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourcePatch {
    pub target: Target,
    /// RFC 6902 operations; paths are relative to the targeted resource.
    pub operations: Vec<json_patch::PatchOperation>,
}

/// Identifies one resource across all sources of a package.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Target {
    pub resource_type: String,
    pub id: Option<String>,
    pub url: Option<String>,
}

impl Target {
    fn matches(&self, resource: &Value) -> bool {
        let field = |name: &str| resource.get(name).and_then(Value::as_str);
        field("resourceType") == Some(self.resource_type.as_str())
            && self.id.as_deref().is_none_or(|id| field("id") == Some(id))
            && self
                .url
                .as_deref()
                .is_none_or(|url| field("url") == Some(url))
    }
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.resource_type)?;
        if let Some(id) = &self.id {
            write!(f, "/{id}")?;
        }
        if let Some(url) = &self.url {
            write!(f, " ({url})")?;
        }
        Ok(())
    }
}

/// What produced a [`Change`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Origin {
    /// A patch file, relative to the manifest directory.
    Patch(PathBuf),
    Rule(&'static str),
}

impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Origin::Patch(path) => write!(f, "patch {}", path.display()),
            Origin::Rule(name) => write!(f, "rule {name}"),
        }
    }
}

/// One edit to an upstream resource. `None` means the value is absent.
#[derive(Debug, Clone)]
pub struct Change {
    /// Upstream file the resource was read from.
    pub source: PathBuf,
    /// `ResourceType/id`.
    pub resource: String,
    /// JSON pointer into the resource.
    pub pointer: String,
    pub before: Option<Value>,
    pub after: Option<Value>,
    pub origin: Origin,
}

/// A file produced by [`build`].
#[derive(Debug)]
pub struct Output {
    pub path: PathBuf,
    pub contents: String,
}

#[derive(Debug, Default)]
pub struct Build {
    pub outputs: Vec<Output>,
    pub changes: Vec<Change>,
}

struct LoadedPatch {
    /// Relative to the manifest directory.
    path: PathBuf,
    file: PatchFile,
}

struct ActiveRule<'a> {
    rule: &'static dyn rules::Rule,
    config: &'a RuleConfig,
}

fn read(path: &Path) -> Result<String, Error> {
    std::fs::read_to_string(path).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn parse_json<T: serde::de::DeserializeOwned>(path: &Path, text: &str) -> Result<T, Error> {
    serde_json::from_str(text).map_err(|source| Error::Json {
        path: path.to_path_buf(),
        source,
    })
}

fn json_files(root: &Path) -> Result<Vec<PathBuf>, Error> {
    let mut files = Vec::new();
    for entry in WalkDir::new(root).sort_by_file_name() {
        let entry = entry.map_err(|e| Error::Io {
            path: root.to_path_buf(),
            source: e.into(),
        })?;
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let is_json = path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"));
        if entry.file_type().is_file() && is_json && !name.ends_with(".min.json") {
            files.push(path.to_path_buf());
        }
    }
    Ok(files)
}

fn load_patches(dir: &Path) -> Result<Vec<LoadedPatch>, Error> {
    json_files(dir)?
        .into_iter()
        .map(|path| {
            let file: PatchFile = parse_json(&path, &read(&path)?)?;
            let relative = path.strip_prefix(dir).unwrap_or(&path).to_path_buf();
            if file
                .patches
                .iter()
                .any(|p| p.target.id.is_none() && p.target.url.is_none())
            {
                return Err(Error::UnaddressedTarget { patch: relative });
            }
            Ok(LoadedPatch {
                path: relative,
                file,
            })
        })
        .collect()
}

fn resource_key(resource: &Value) -> String {
    let field = |name: &str| resource.get(name).and_then(Value::as_str).unwrap_or("?");
    format!("{}/{}", field("resourceType"), field("id"))
}

/// Value at `pointer`, resolving a trailing `-` (JSON Patch "end of array") to
/// the last element so an append can be reported where it landed.
fn lookup(doc: &Value, pointer: &str) -> (String, Option<Value>) {
    if let Some(parent) = pointer.strip_suffix("/-")
        && let Some(array) = doc.pointer(parent).and_then(Value::as_array)
        && let Some(last) = array.len().checked_sub(1)
    {
        let resolved = format!("{parent}/{last}");
        let value = doc.pointer(&resolved).cloned();
        return (resolved, value);
    }
    (pointer.to_string(), doc.pointer(pointer).cloned())
}

struct Context<'a> {
    source: &'a Path,
    patches: &'a [LoadedPatch],
    rules: &'a [ActiveRule<'a>],
    /// Resources matched per (patch file, patch) so stale targets are reported.
    matches: &'a mut HashMap<(usize, usize), usize>,
    changes: &'a mut Vec<Change>,
}

fn process_resource(resource: &mut Value, ctx: &mut Context<'_>) -> Result<(), Error> {
    let key = resource_key(resource);

    for (file_index, patch_file) in ctx.patches.iter().enumerate() {
        for (patch_index, patch) in patch_file.file.patches.iter().enumerate() {
            if !patch.target.matches(resource) {
                continue;
            }
            *ctx.matches.entry((file_index, patch_index)).or_default() += 1;

            for operation in &patch.operations {
                let path = operation.path().to_string();
                let before = resource.pointer(&path).cloned();
                json_patch::patch(resource, std::slice::from_ref(operation)).map_err(|source| {
                    Error::Apply {
                        patch: patch_file.path.clone(),
                        resource: key.clone(),
                        source,
                    }
                })?;
                if matches!(operation, json_patch::PatchOperation::Test(_)) {
                    continue;
                }
                let (pointer, after) = lookup(resource, &path);
                ctx.changes.push(Change {
                    source: ctx.source.to_path_buf(),
                    resource: key.clone(),
                    pointer,
                    before,
                    after,
                    origin: Origin::Patch(patch_file.path.clone()),
                });
            }
        }
    }

    let resource_type = resource
        .get("resourceType")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    for active in ctx.rules {
        let types = &active.config.resource_types;
        if !types.is_empty() && !types.contains(&resource_type) {
            continue;
        }
        for edit in active.rule.apply(resource) {
            ctx.changes.push(Change {
                source: ctx.source.to_path_buf(),
                resource: key.clone(),
                pointer: edit.pointer,
                before: Some(edit.before),
                after: Some(edit.after),
                origin: Origin::Rule(active.rule.name()),
            });
        }
    }

    minimize::minimize_resource(resource);
    Ok(())
}

/// Resolves `.` and `..` in `path` without touching the filesystem, so
/// reported source paths read `pkg/definitions/x.json` rather than
/// `pkg/patches/../definitions/x.json`.
fn normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir
                if matches!(
                    normalized.components().next_back(),
                    Some(Component::Normal(_))
                ) =>
            {
                normalized.pop();
            }
            other => normalized.push(other),
        }
    }
    normalized
}

/// Output path for an upstream file: `foo.json` becomes `foo.min.json`.
fn output_path(source: &Path) -> PathBuf {
    source.with_extension("min.json")
}

/// Reads the manifest at `manifest_path`, applies its patches and rules to the
/// upstream sources, and returns the files to write and every change made.
/// Nothing is written to disk.
///
/// # Errors
///
/// Returns an error if the manifest, a patch file or a source cannot be read or
/// parsed, if the manifest names an unknown rule, if a patch operation fails,
/// or if a patch target does not match exactly one resource.
pub fn build(manifest_path: &Path) -> Result<Build, Error> {
    let dir = manifest_path.parent().unwrap_or(Path::new("."));
    let manifest: Manifest =
        toml::from_str(&read(manifest_path)?).map_err(|source| Error::Manifest {
            path: manifest_path.to_path_buf(),
            source: Box::new(source),
        })?;

    let rules = manifest
        .rules
        .iter()
        .map(|config| {
            rules::find(&config.name)
                .map(|rule| ActiveRule { rule, config })
                .ok_or_else(|| Error::UnknownRule(config.name.clone()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let patches = load_patches(dir)?;

    let mut build = Build::default();
    let mut matches = HashMap::new();

    for source in &manifest.sources {
        let source = normalize(&dir.join(source));
        let files = if source.is_dir() {
            json_files(&source)?
        } else {
            vec![source]
        };

        for file in files {
            let mut document: Value = parse_json(&file, &read(&file)?)?;
            // Not every JSON file in an upstream package is a resource
            // (e.g. fhir.schema.json); those are left alone.
            let Some(resource_type) = document.get("resourceType").and_then(Value::as_str) else {
                continue;
            };

            let mut ctx = Context {
                source: &file,
                patches: &patches,
                rules: &rules,
                matches: &mut matches,
                changes: &mut build.changes,
            };
            if resource_type == "Bundle" {
                let entries = document
                    .get_mut("entry")
                    .and_then(Value::as_array_mut)
                    .into_iter()
                    .flatten();
                for resource in entries.filter_map(|entry| entry.get_mut("resource")) {
                    process_resource(resource, &mut ctx)?;
                }
            } else {
                process_resource(&mut document, &mut ctx)?;
            }

            build.outputs.push(Output {
                path: output_path(&file),
                contents: serde_json::to_string(&document).map_err(|source| Error::Json {
                    path: file.clone(),
                    source,
                })?,
            });
        }
    }

    for (file_index, patch_file) in patches.iter().enumerate() {
        for (patch_index, patch) in patch_file.file.patches.iter().enumerate() {
            let count = matches
                .get(&(file_index, patch_index))
                .copied()
                .unwrap_or(0);
            if count != 1 {
                return Err(Error::TargetMatchCount {
                    patch: patch_file.path.clone(),
                    target: patch.target.clone(),
                    count,
                });
            }
        }
    }

    Ok(build)
}
