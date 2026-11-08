//! Validates every example resource an implementation guide ships against the
//! profiles it claims in `meta.profile`.
//!
//! The in-repo packages (US Core 7.0.0 and CARIN Blue Button) are always
//! measured. More packages (a Da Vinci guide, say) are added with
//! `HASTE_PROFILING_CORPUS_DIRS`, a colon-separated list of package
//! directories that hold `StructureDefinition-*.json`, `ValueSet-*.json` and an
//! `example/` folder.
//!
//! `HASTE_PROFILING_FINGERPRINT=<file>` writes one line per example with the
//! outcome (issue codes, diagnostics and locations), to diff two builds.
use criterion::{Criterion, criterion_group, criterion_main};
use haste_fhir_client::canonical_resolver::CanonicalResolver;
use haste_fhir_model::r4::generated::resources::{Bundle, Resource, ResourceType};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_profiling::{FHIRProfileArguments, validate_profile_by_url};
use haste_reflect::MetaValue;
use std::{
    collections::HashMap,
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};

fn artifacts() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../artifacts/r4")
}

/// Every canonical resource of every loaded package, by URL (version stripped).
struct Canonicals(HashMap<String, Arc<Resource>>);

impl CanonicalResolver for Canonicals {
    fn resolve(
        &self,
        _resource_type: ResourceType,
        canonical_url: &str,
    ) -> impl Future<Output = Result<Option<Arc<Resource>>, OperationOutcomeError>> + Send {
        let url = canonical_url.split('|').next().unwrap_or_default();
        std::future::ready(Ok(self.0.get(url).cloned()))
    }
}

fn read_resource(path: &Path) -> Option<Resource> {
    let text = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

fn url_of(resource: &Resource) -> Option<String> {
    resource
        .get_field("url")
        .and_then(|url| url.get_field("value"))
        .and_then(|value| value.as_any().downcast_ref::<String>())
        .cloned()
}

fn load_bundle(canonicals: &mut HashMap<String, Arc<Resource>>, path: &Path) {
    let bundle: Bundle = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    for resource in bundle
        .entry
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.resource)
    {
        if let Some(url) = url_of(&resource) {
            canonicals.entry(url).or_insert_with(|| Arc::new(*resource));
        }
    }
}

fn load_package(canonicals: &mut HashMap<String, Arc<Resource>>, dir: &Path) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if !(name.starts_with("StructureDefinition-") || name.starts_with("ValueSet-"))
            || path.extension().is_none_or(|extension| extension != "json")
        {
            continue;
        }
        if let Some(resource) = read_resource(&path)
            && let Some(url) = url_of(&resource)
        {
            // A package's own definition wins over the base ones.
            canonicals.insert(url, Arc::new(resource));
        }
    }
}

struct Example {
    name: String,
    profiles: Vec<String>,
    resource: Resource,
}

fn load_examples(dir: &Path) -> Vec<Example> {
    let mut examples: Vec<Example> = std::fs::read_dir(dir.join("example"))
        .unwrap()
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            let resource = read_resource(&path)?;
            let profiles: Vec<String> =
                serde_json::from_str::<serde_json::Value>(&std::fs::read_to_string(&path).unwrap())
                    .ok()?
                    .pointer("/meta/profile")?
                    .as_array()?
                    .iter()
                    .filter_map(|p| p.as_str().map(str::to_string))
                    .collect();
            Some(Example {
                name: path.file_stem().unwrap().to_string_lossy().into_owned(),
                profiles,
                resource,
            })
        })
        .collect();
    examples.sort_by(|a, b| a.name.cmp(&b.name));
    examples
}

/// The resolver over the base definitions and every package, with the
/// packages to measure.
fn corpus() -> (Arc<Canonicals>, Vec<(String, PathBuf)>) {
    let base = artifacts();
    let mut canonicals = HashMap::new();
    for bundle in [
        "profiles-types.min.json",
        "profiles-resources.min.json",
        "profiles-others.min.json",
        "extension-definitions.min.json",
        "valuesets.min.json",
    ] {
        load_bundle(
            &mut canonicals,
            &base.join("hl7-core/definitions/hl7").join(bundle),
        );
    }

    let mut packages: Vec<(String, PathBuf)> = vec![
        ("us-core-7".to_string(), base.join("hl7.fhir.us.core-7.0.0")),
        ("carin-bb".to_string(), base.join("hl7.fhir.us.carin-bb")),
    ];
    if let Ok(extra) = std::env::var("HASTE_PROFILING_CORPUS_DIRS") {
        for dir in extra.split(':').filter(|dir| !dir.is_empty()) {
            let path = PathBuf::from(dir);
            let name = path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default();
            packages.push((name, path));
        }
    }
    for (_, dir) in &packages {
        load_package(&mut canonicals, dir);
    }
    (Arc::new(Canonicals(canonicals)), packages)
}

fn bench(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let (resolver, packages) = corpus();

    let mut fingerprint = std::env::var("HASTE_PROFILING_FINGERPRINT")
        .ok()
        .map(|path| std::fs::File::create(path).unwrap());

    for (name, dir) in &packages {
        let examples = load_examples(dir);
        let mut ok = 0usize;
        let mut errors = 0usize;
        let mut issues = 0usize;
        for example in &examples {
            for profile in &example.profiles {
                let result = rt.block_on(validate_profile_by_url(
                    FHIRProfileArguments::new(resolver.clone()),
                    profile,
                    &example.resource,
                ));
                let line = match &result {
                    Ok(outcome) => {
                        ok += 1;
                        issues += outcome.issue.len();
                        let mut parts: Vec<String> = outcome
                            .issue
                            .iter()
                            .map(|issue| {
                                format!(
                                    "{}|{}|{}",
                                    issue.code.as_str().unwrap_or(""),
                                    issue
                                        .location
                                        .as_ref()
                                        .and_then(|l| l.first())
                                        .and_then(|l| l.value.as_deref())
                                        .unwrap_or(""),
                                    issue
                                        .diagnostics
                                        .as_ref()
                                        .and_then(|d| d.value.as_deref())
                                        .unwrap_or("")
                                )
                            })
                            .collect();
                        parts.sort();
                        format!("ok {}", parts.join(" ;; "))
                    }
                    Err(error) => {
                        errors += 1;
                        format!("err {error}")
                    }
                };
                if let Some(file) = fingerprint.as_mut() {
                    writeln!(file, "{name}/{} @ {profile}: {line}", example.name).unwrap();
                }
            }
        }
        eprintln!(
            "{name}: examples={} validations_ok={ok} validations_err={errors} issues={issues}",
            examples.len()
        );

        c.bench_function(&format!("corpus_{name}"), |b| {
            b.iter(|| {
                for example in &examples {
                    for profile in &example.profiles {
                        let _ = rt.block_on(validate_profile_by_url(
                            FHIRProfileArguments::new(resolver.clone()),
                            profile,
                            &example.resource,
                        ));
                    }
                }
            });
        });
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);
