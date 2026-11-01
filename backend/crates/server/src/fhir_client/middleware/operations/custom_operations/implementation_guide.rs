//! `ImplementationGuide/$install` and `ImplementationGuide/$uninstall`: FHIR
//! packages (implementation guides) installed into a project.
//!
//! A package's conformance resources are written to the project under their
//! package ids, each tagged with the package: [`PACKAGE_TAG_SYSTEM`], the
//! package name as the code and its version as the coding's version. The
//! package itself is recorded as an `ImplementationGuide` whose id is the
//! package name and whose `definition.resource` lists what was written. A
//! later install or uninstall reads that record to find what to replace or
//! delete, so neither depends on the search index having caught up.
//!
//! Everything one install writes or deletes goes through the caller's client
//! in one transaction: access control, plan limits and auditing apply to
//! each resource, and a failure leaves the project as it was.

use crate::fhir_client::{
    middleware::{Backend, operations::ServerOperationContext},
    resolver,
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use haste_fhir_client::request::InvocationRequest;
use haste_fhir_generated_ops::generated::{
    HasteHealthImplementationguideInstall as Install,
    HasteHealthImplementationguideUninstall as Uninstall,
};
use haste_fhir_model::r4::generated::{
    resources::{
        Bundle, BundleEntry, BundleEntryRequest, ImplementationGuide,
        ImplementationGuideDefinition, ImplementationGuideDefinitionResource,
        ImplementationGuideDefinitionResourceExampleTypeChoice, ImplementationGuideDependsOn,
        OperationOutcome, OperationOutcomeIssue, Resource, ResourceType,
    },
    terminology::{
        BoundCode, BundleType, FHIRVersion, HttpVerb, IssueSeverity, IssueType, PublicationStatus,
    },
    types::{
        Coding, FHIRBoolean, FHIRCanonical, FHIRCode, FHIRId, FHIRInteger, FHIRMarkdown,
        FHIRString, FHIRUri, Meta, Reference,
    },
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_ops::OperationExecutor;
use haste_fhir_package::{Limits, Manifest, Package, PackageFile, PackageSpec};
use haste_jwt::{ProjectId, ResourceId, TenantId};
use haste_reflect::MetaValue;
use haste_repository::fhir::FHIRRepository;
use std::collections::{HashMap, HashSet, VecDeque};

/// The `meta.tag` system marking a resource as installed from a package.
pub const PACKAGE_TAG_SYSTEM: &str = "https://haste.health/fhir/CodeSystem/fhir-package";

/// The conformance resource types installed from a package. HAPI's default
/// set, without Subscription: an installed Subscription would start sending
/// notifications.
const INSTALL_TYPES: [ResourceType; 6] = [
    ResourceType::StructureDefinition,
    ResourceType::ValueSet,
    ResourceType::CodeSystem,
    ResourceType::SearchParameter,
    ResourceType::ConceptMap,
    ResourceType::NamingSystem,
];

/// Example resource types never installed, as they would change what the
/// server does rather than add data: custom operations, subscriptions,
/// search parameters, and Haste Health's own administrative types.
const EXAMPLE_SKIP_TYPES: [ResourceType; 14] = [
    ResourceType::OperationDefinition,
    ResourceType::Subscription,
    ResourceType::SearchParameter,
    ResourceType::ImplementationGuide,
    ResourceType::AccessPolicyV2,
    ResourceType::AccessPolicyV2Assignment,
    ResourceType::AccessPolicyV3,
    ResourceType::AccessPolicyV3Assignment,
    ResourceType::ClientApplication,
    ResourceType::DeidentificationPolicy,
    ResourceType::IdentityProvider,
    ResourceType::Membership,
    ResourceType::Project,
    ResourceType::User,
];

/// The most warnings one response lists; the rest are counted.
const MAX_ISSUES: usize = 200;

/// The FHIR core packages (`hl7.fhir.r4.core`, `hl7.fhir.r5.core`, ...) and
/// their companions: the server has R4's built in, and the others are for
/// other FHIR versions.
fn is_core_package(name: &str) -> bool {
    name.starts_with("hl7.fhir.r")
        && [".core", ".examples", ".expansions", ".elements"]
            .iter()
            .any(|suffix| name.ends_with(suffix))
}

/// Whether a package is for FHIR R4, the version this server serves. A
/// package that does not say is given the benefit of the doubt.
fn is_for_r4(manifest: &Manifest) -> bool {
    manifest.fhir_versions.is_empty()
        || manifest
            .fhir_versions
            .iter()
            .any(|version| version.starts_with("4.0") || version.eq_ignore_ascii_case("r4"))
}

/// The id of the `ImplementationGuide` recording package `name`: the name,
/// limited to the characters and length an id allows.
fn record_id(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '.') {
                c
            } else {
                '-'
            }
        })
        .take(64)
        .collect()
}

fn fhir_string(value: impl Into<String>) -> Box<FHIRString> {
    Box::new(FHIRString {
        value: Some(value.into()),
        ..Default::default()
    })
}

fn package_coding(manifest: &Manifest) -> Coding {
    Coding {
        system: Some(Box::new(FHIRUri {
            value: Some(PACKAGE_TAG_SYSTEM.to_string()),
            ..Default::default()
        })),
        version: Some(fhir_string(manifest.version.clone())),
        code: Some(Box::new(FHIRCode {
            value: Some(manifest.name.clone()),
            ..Default::default()
        })),
        ..Default::default()
    }
}

fn meta(resource: &Resource) -> Option<&Meta> {
    resource
        .get_field("meta")
        .and_then(|meta| meta.as_any().downcast_ref::<Box<Meta>>())
        .map(AsRef::as_ref)
}

/// The package that installed `resource`, from its tag.
fn installed_by(resource: &Resource) -> Option<&str> {
    package_tag(meta(resource))
}

/// The package named by a package tag in `meta`.
fn package_tag(meta: Option<&Meta>) -> Option<&str> {
    meta?
        .tag
        .as_ref()?
        .iter()
        .find(|tag| {
            tag.system
                .as_ref()
                .and_then(|system| system.value.as_deref())
                == Some(PACKAGE_TAG_SYSTEM)
        })
        .and_then(|tag| tag.code.as_ref())
        .and_then(|code| code.value.as_deref())
}

fn canonical_url(resource: &Resource) -> Option<&str> {
    resource
        .get_field("url")
        .and_then(|url| url.as_any().downcast_ref::<Box<FHIRUri>>())
        .and_then(|url| url.value.as_deref())
}

/// Tags `resource` as installed from the package, replacing any package tag
/// it came with.
fn tag(resource: &mut Resource, manifest: &Manifest) -> Result<(), OperationOutcomeError> {
    let meta = resource
        .get_field_mut("meta")
        .and_then(|meta| {
            let meta: &mut dyn std::any::Any = meta;
            meta.downcast_mut::<Option<Box<Meta>>>()
        })
        .ok_or_else(|| {
            OperationOutcomeError::fatal(
                IssueType::exception(),
                "A package resource has no meta element.".to_string(),
            )
        })?;
    let tags = meta
        .get_or_insert_with(Box::default)
        .tag
        .get_or_insert_with(Vec::new);
    tags.retain(|tag| {
        tag.system
            .as_ref()
            .and_then(|system| system.value.as_deref())
            != Some(PACKAGE_TAG_SYSTEM)
    });
    tags.push(package_coding(manifest));
    Ok(())
}

fn entry(method: BoundCode<HttpVerb>, key: &Key, resource: Option<Resource>) -> BundleEntry {
    BundleEntry {
        resource: resource.map(Box::new),
        request: Some(BundleEntryRequest {
            method,
            url: Box::new(format!("{}/{}", key.0.as_ref(), key.1).into()),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// A resource in the project: its type and id.
type Key = (ResourceType, String);

/// The resources a package record lists in `definition.resource`.
fn record_resources(record: &ImplementationGuide) -> Vec<Key> {
    record
        .definition
        .iter()
        .flat_map(|definition| &definition.resource)
        .filter_map(|resource| resource.reference.reference.as_ref()?.value.as_deref())
        .filter_map(|reference| {
            let (resource_type, id) = reference.split_once('/')?;
            Some((ResourceType::try_from(resource_type).ok()?, id.to_string()))
        })
        .collect()
}

/// The project's record of package `name`, if it is installed.
async fn installed_record<B: Backend>(
    context: &ServerOperationContext<B>,
    tenant: &TenantId,
    project: &ProjectId,
    name: &str,
) -> Result<Option<ImplementationGuide>, OperationOutcomeError> {
    let record = context
        .state
        .repo
        .read_latest(
            tenant,
            project,
            &ResourceType::ImplementationGuide,
            &ResourceId::new(record_id(name)),
        )
        .await?;
    Ok(match record {
        Some(Resource::ImplementationGuide(record))
            if record.packageId.value.as_deref() == Some(name)
                && package_tag(record.meta.as_deref()) == Some(name) =>
        {
            Some(record)
        }
        _ => None,
    })
}

/// The current version of each of `keys` in the project, by key.
async fn read_existing<B: Backend>(
    context: &ServerOperationContext<B>,
    tenant: &TenantId,
    project: &ProjectId,
    keys: Vec<Key>,
) -> Result<HashMap<Key, Resource>, OperationOutcomeError> {
    let keys = keys
        .into_iter()
        .collect::<HashSet<_>>()
        .into_iter()
        .map(|(resource_type, id)| (resource_type, ResourceId::new(id)))
        .collect::<Vec<_>>();
    let mut existing = HashMap::new();
    for chunk in keys.chunks(1000) {
        for resource in context
            .state
            .repo
            .read_latest_many(tenant, project, chunk)
            .await?
        {
            if let Some(id) = resource.id().clone() {
                existing.insert((resource.resource_type(), id), resource);
            }
        }
    }
    Ok(existing)
}

/// What an install reports back.
#[derive(Default)]
struct Report {
    packages: Vec<Install::OutputPackage>,
    issues: Vec<OperationOutcomeIssue>,
    omitted_issues: usize,
}

impl Report {
    fn warn(&mut self, diagnostic: String) {
        if self.issues.len() >= MAX_ISSUES {
            self.omitted_issues += 1;
            return;
        }
        self.issues.push(OperationOutcomeIssue {
            severity: IssueSeverity::warning(),
            code: IssueType::informational(),
            diagnostics: Some(fhir_string(diagnostic)),
            ..Default::default()
        });
    }

    fn skip(&mut self, name: &str, version: &str, reason: String) {
        self.packages.push(Install::OutputPackage {
            name: *fhir_string(name),
            version: *fhir_string(version),
            status: FHIRCode {
                value: Some("skipped".to_string()),
                ..Default::default()
            },
            previousVersion: None,
            resources: None,
            reason: Some(*fhir_string(reason)),
        });
    }
}

/// A package chosen for installation.
struct Selected {
    package: Package,
    /// Named or uploaded by the caller, rather than pulled in as a
    /// dependency. Only these have their examples installed.
    requested: bool,
    /// The project's record of the version installed before, if any.
    previous: Option<ImplementationGuide>,
}

/// Where a package to consider comes from.
enum Source {
    Uploaded(Package),
    Named(PackageSpec),
    Dependency { name: String, version: String },
}

/// Fetches `name` from the configured registry.
async fn fetch<B: Backend>(
    context: &ServerOperationContext<B>,
    name: &str,
    version: Option<&str>,
    limits: &Limits,
) -> Result<Package, OperationOutcomeError> {
    let config = &context.state.config.packages;
    let version = haste_fhir_package::resolve_version(&config.registry_url, name, version).await?;
    let bytes = haste_fhir_package::download(
        &config.registry_url,
        name,
        &version,
        config.max_package_bytes,
    )
    .await?;
    Ok(haste_fhir_package::read(&bytes, limits)?)
}

fn decode_upload(
    content: &str,
    max_bytes: u64,
    limits: &Limits,
) -> Result<Package, OperationOutcomeError> {
    let content = content
        .chars()
        .filter(|c| !c.is_ascii_whitespace())
        .collect::<String>();
    // Base64 is 4 characters for 3 bytes.
    if (content.len() as u64) / 4 * 3 > max_bytes {
        return Err(haste_fhir_package::PackageError::TooLarge(max_bytes).into());
    }
    let bytes = STANDARD.decode(content).map_err(|e| {
        OperationOutcomeError::error(
            IssueType::invalid(),
            format!("npmContent is not valid base64: {e}"),
        )
    })?;
    let package = haste_fhir_package::read(&bytes, limits)?;
    if !haste_fhir_package::valid_name(&package.manifest.name) {
        return Err(
            haste_fhir_package::PackageError::InvalidName(package.manifest.name.clone()).into(),
        );
    }
    if !haste_fhir_package::valid_version(&package.manifest.version) {
        return Err(haste_fhir_package::PackageError::InvalidVersion(
            package.manifest.version.clone(),
        )
        .into());
    }
    Ok(package)
}

/// The packages to install: the ones asked for, then (when `dependencies`)
/// the packages they depend on, breadth first, so the version a package
/// nearer the top asks for wins.
async fn select<B: Backend>(
    context: &ServerOperationContext<B>,
    tenant: &TenantId,
    project: &ProjectId,
    roots: Vec<Source>,
    dependencies: bool,
    report: &mut Report,
) -> Result<Vec<Selected>, OperationOutcomeError> {
    let config = &context.state.config.packages;
    let limits = Limits {
        max_uncompressed_bytes: config.max_uncompressed_bytes,
        max_files: config.max_files,
    };

    let mut uploaded = HashMap::new();
    let mut queue = VecDeque::new();
    for root in roots {
        match root {
            Source::Uploaded(package) => {
                uploaded.insert(package.manifest.name.clone(), package.clone());
                queue.push_back(Source::Uploaded(package));
            }
            source => queue.push_back(source),
        }
    }

    let mut selected: Vec<Selected> = Vec::new();
    let mut seen = HashSet::new();
    while let Some(source) = queue.pop_front() {
        let (package, requested) = match source {
            Source::Uploaded(package) => {
                if !seen.insert(package.manifest.name.clone()) {
                    continue;
                }
                (package, true)
            }
            Source::Named(spec) => {
                if !seen.insert(spec.name.clone()) {
                    continue;
                }
                if is_core_package(&spec.name) {
                    report.skip(
                        &spec.name,
                        spec.version.as_deref().unwrap_or_default(),
                        "The FHIR core package is built into the server.".to_string(),
                    );
                    continue;
                }
                let package = match uploaded.remove(&spec.name) {
                    Some(package) => package,
                    None => fetch(context, &spec.name, spec.version.as_deref(), &limits).await?,
                };
                (package, true)
            }
            Source::Dependency { name, version } => {
                if !seen.insert(name.clone()) {
                    continue;
                }
                if is_core_package(&name) {
                    continue;
                }
                if config.skip_dependencies.contains(&name) {
                    report.skip(
                        &name,
                        &version,
                        "Not installed as a dependency, as it is a large terminology package. \
                         Install it by name to include it."
                            .to_string(),
                    );
                    continue;
                }
                if let Some(record) = installed_record(context, tenant, project, &name).await? {
                    let installed = record
                        .version
                        .as_ref()
                        .and_then(|version| version.value.clone())
                        .unwrap_or_default();
                    report.skip(
                        &name,
                        &installed,
                        format!("Already installed (version {installed})."),
                    );
                    continue;
                }
                let package = match uploaded.remove(&name) {
                    Some(package) => package,
                    None => match fetch(context, &name, Some(&version), &limits).await {
                        Ok(package) => package,
                        Err(error) => {
                            let reason = error
                                .outcome()
                                .issue
                                .first()
                                .and_then(|issue| issue.diagnostics.as_ref())
                                .and_then(|diagnostics| diagnostics.value.clone())
                                .unwrap_or_else(|| "It could not be fetched.".to_string());
                            report.skip(&name, &version, reason);
                            continue;
                        }
                    },
                };
                (package, false)
            }
        };

        if !is_for_r4(&package.manifest) {
            let reason = format!(
                "It is for FHIR {}; this server serves FHIR R4 (4.0.1).",
                package.manifest.fhir_versions.join(", ")
            );
            if requested {
                return Err(OperationOutcomeError::error(
                    IssueType::not_supported(),
                    format!(
                        "Package '{}' cannot be installed. {reason}",
                        package.manifest.name
                    ),
                ));
            }
            report.skip(&package.manifest.name, &package.manifest.version, reason);
            continue;
        }

        if dependencies {
            for (name, version) in &package.manifest.dependencies {
                queue.push_back(Source::Dependency {
                    name: name.clone(),
                    version: version.clone(),
                });
            }
        }

        let previous = installed_record(context, tenant, project, &package.manifest.name).await?;
        selected.push(Selected {
            package,
            requested,
            previous,
        });
    }

    Ok(selected)
}

/// A resource to write for a package.
struct Write {
    key: Key,
    resource: Resource,
    example: bool,
}

/// The resources of `selected` to install, and its own ImplementationGuide
/// if it ships one. Files that cannot be read, or that another package in
/// this install already wrote, are reported and left out.
fn package_writes(
    selected: &Selected,
    examples: bool,
    claimed: &mut HashMap<Key, String>,
    report: &mut Report,
) -> (Vec<Write>, Option<ImplementationGuide>) {
    let manifest = &selected.package.manifest;
    let label = format!("{}@{}", manifest.name, manifest.version);
    let example_files: &[PackageFile] = if examples && selected.requested {
        &selected.package.examples
    } else {
        &[]
    };
    let files = selected
        .package
        .resources
        .iter()
        .map(|file| (file, false))
        .chain(example_files.iter().map(|file| (file, true)));

    let mut own_guide = None;
    let mut writes = Vec::new();
    let mut retargeted = HashSet::new();
    for (file, example) in files {
        let Some(resource) = parse_file(file, example, &label, &mut retargeted, report) else {
            continue;
        };
        let resource_type = resource.resource_type();
        if let Resource::ImplementationGuide(guide) = resource {
            own_guide.get_or_insert(guide);
            continue;
        }

        let Some(id) = resource.id().clone() else {
            report.warn(format!("{label}: '{}' has no id.", file.path));
            continue;
        };
        let key = (resource_type, id);
        if let Some(other) = claimed.get(&key) {
            report.warn(format!(
                "{label}: {}/{} was not installed, as {other} has a resource with that id.",
                key.0.as_ref(),
                key.1
            ));
            continue;
        }
        claimed.insert(key.clone(), label.clone());
        writes.push(Write {
            key,
            resource,
            example,
        });
    }
    if !retargeted.is_empty() {
        let mut versions = retargeted.into_iter().collect::<Vec<_>>();
        versions.sort();
        report.warn(format!(
            "{label}: StructureDefinitions that name FHIR {} were installed as FHIR 4.0.1, \
             the version the package is for.",
            versions.join(", ")
        ));
    }
    (writes, own_guide)
}

/// Enough of a file to decide whether to read the rest.
#[derive(serde::Deserialize)]
struct FileHead {
    #[serde(rename = "resourceType")]
    resource_type: Option<String>,
    #[serde(rename = "fhirVersion")]
    fhir_version: Option<serde_json::Value>,
}

/// The resource in `file`, when it is one an install writes: an installed
/// conformance type (or the package's ImplementationGuide), or an example
/// of a type examples may have. Anything else is skipped without reading it
/// in full, and a file that cannot be read is reported.
///
/// A StructureDefinition naming a FHIR version R4 does not have is read as
/// 4.0.1, the version of the package it is in (`retargeted` collects the
/// versions named). `hl7.fhir.uv.extensions.r4@1.0.0`, which US Core 6
/// depends on, labels every extension 5.0.0.
fn parse_file(
    file: &PackageFile,
    example: bool,
    label: &str,
    retargeted: &mut HashSet<String>,
    report: &mut Report,
) -> Option<Resource> {
    let unreadable = |report: &mut Report, error: &dyn std::fmt::Display| {
        report.warn(format!(
            "{label}: '{}' could not be read: {error}",
            file.path
        ));
    };
    let head = match serde_json::from_slice::<FileHead>(&file.content) {
        Ok(head) => head,
        Err(error) => {
            unreadable(report, &error);
            return None;
        }
    };
    // Not a resource (an index, say), or a type from another FHIR version.
    let resource_type = ResourceType::try_from(head.resource_type?.as_str()).ok()?;
    let wanted = if example {
        !EXAMPLE_SKIP_TYPES.contains(&resource_type)
    } else {
        INSTALL_TYPES.contains(&resource_type) || resource_type == ResourceType::ImplementationGuide
    };
    if !wanted {
        return None;
    }

    let foreign_version = match &head.fhir_version {
        Some(serde_json::Value::String(version))
            if resource_type == ResourceType::StructureDefinition
                && BoundCode::<FHIRVersion>::new(version).is_none() =>
        {
            Some(version.clone())
        }
        _ => None,
    };
    let parsed = match &foreign_version {
        None => serde_json::from_slice::<Resource>(&file.content),
        Some(_) => {
            serde_json::from_slice::<serde_json::Value>(&file.content).and_then(|mut value| {
                value["fhirVersion"] = serde_json::Value::String("4.0.1".to_string());
                serde_json::from_value::<Resource>(value)
            })
        }
    };
    match parsed {
        Ok(resource) => {
            retargeted.extend(foreign_version);
            Some(resource)
        }
        // A package's own ImplementationGuide only seeds its record, which
        // is made from package.json when it cannot be read.
        Err(_) if resource_type == ResourceType::ImplementationGuide => None,
        Err(error) => {
            unreadable(report, &error);
            None
        }
    }
}

/// Whether installing `write` for package `name` may replace `existing`:
/// the package installed it, or nothing did and it is the same canonical
/// resource (loaded by hand, say).
fn may_replace(existing: &Resource, write: &Write, name: &str) -> Result<(), String> {
    match installed_by(existing) {
        Some(owner) if owner == name => Ok(()),
        Some(owner) => Err(format!(
            "package '{owner}' installed a resource with that id"
        )),
        None if canonical_url(existing).is_some()
            && canonical_url(existing) == canonical_url(&write.resource) =>
        {
            Ok(())
        }
        None => Err("the project already has a resource with that id".to_string()),
    }
}

/// The ImplementationGuide recording an installed package: the package's own
/// ImplementationGuide when it ships one, otherwise one made from its
/// package.json, listing the resources installed.
fn package_record(
    manifest: &Manifest,
    own_guide: Option<ImplementationGuide>,
    written: &[(Key, bool)],
    canonicals: &HashMap<String, String>,
) -> ImplementationGuide {
    let mut record = own_guide.unwrap_or_else(|| ImplementationGuide {
        url: Box::new(FHIRUri {
            value: Some(match &manifest.canonical {
                Some(canonical) => format!(
                    "{}/ImplementationGuide/{}",
                    canonical.trim_end_matches('/'),
                    manifest.name
                ),
                None => format!("urn:fhir-package:{}", manifest.name),
            }),
            ..Default::default()
        }),
        name: fhir_string(
            manifest
                .name
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
                .collect::<String>(),
        ),
        status: PublicationStatus::active(),
        dependsOn: Some(
            manifest
                .dependencies
                .iter()
                .filter_map(|(name, version)| {
                    Some(ImplementationGuideDependsOn {
                        uri: Box::new(FHIRCanonical {
                            value: Some(canonicals.get(name)?.clone()),
                            ..Default::default()
                        }),
                        packageId: Some(Box::new(FHIRId {
                            value: Some(name.clone()),
                            ..Default::default()
                        })),
                        version: Some(fhir_string(version.clone())),
                        ..Default::default()
                    })
                })
                .collect::<Vec<_>>(),
        )
        .filter(|depends_on| !depends_on.is_empty()),
        ..Default::default()
    });

    record.id = Some(record_id(&manifest.name));
    record.meta = Some(Box::new(Meta {
        tag: Some(vec![package_coding(manifest)]),
        ..Default::default()
    }));
    record.text = None;
    record.manifest = None;
    record.packageId = Box::new(FHIRId {
        value: Some(manifest.name.clone()),
        ..Default::default()
    });
    record.version = Some(fhir_string(manifest.version.clone()));
    if record.title.is_none() {
        record.title = manifest.title.clone().map(fhir_string);
    }
    if record.description.is_none() {
        record.description = manifest.description.clone().map(|description| {
            Box::new(FHIRMarkdown {
                value: Some(description),
                ..Default::default()
            })
        });
    }
    if record.fhirVersion.is_empty() {
        record.fhirVersion = manifest
            .fhir_versions
            .iter()
            .filter_map(|version| BoundCode::new(version))
            .collect();
    }
    if record.fhirVersion.is_empty() {
        record.fhirVersion = BoundCode::new("4.0.1").into_iter().collect();
    }
    record.definition = if written.is_empty() {
        None
    } else {
        Some(ImplementationGuideDefinition {
            resource: written
                .iter()
                .map(
                    |((resource_type, id), example)| ImplementationGuideDefinitionResource {
                        reference: Box::new(Reference {
                            reference: Some(fhir_string(format!(
                                "{}/{id}",
                                resource_type.as_ref()
                            ))),
                            ..Default::default()
                        }),
                        example: Some(
                            ImplementationGuideDefinitionResourceExampleTypeChoice::Boolean(
                                Box::new(FHIRBoolean {
                                    value: Some(*example),
                                    ..Default::default()
                                }),
                            ),
                        ),
                        ..Default::default()
                    },
                )
                .collect(),
            ..Default::default()
        })
    };
    record
}

fn version_of(record: &ImplementationGuide) -> Option<String> {
    record
        .version
        .as_ref()
        .and_then(|version| version.value.clone())
}

/// Writes the examples as a batch, after the conformance resources they
/// may claim as profiles: an example is data, so one the server refuses
/// (one its profile's validation rejects, say) is reported and left out
/// rather than failing the install. `owners` names each entry's package
/// report, whose resource count drops for each example refused.
async fn write_examples<B: Backend>(
    context: &ServerOperationContext<B>,
    entries: Vec<BundleEntry>,
    owners: &[(usize, String, Key)],
    installed: &mut [Install::OutputPackage],
    report: &mut Report,
) -> Result<(), OperationOutcomeError> {
    let response = context
        .ctx
        .client
        .batch(
            context.ctx.clone(),
            Bundle {
                type_: BundleType::batch(),
                entry: Some(entries),
                ..Default::default()
            },
        )
        .await?;
    for ((owner, label, key), entry) in owners.iter().zip(response.entry.unwrap_or_default()) {
        let Some(Resource::OperationOutcome(outcome)) = entry
            .response
            .as_ref()
            .and_then(|response| response.outcome.as_deref())
        else {
            continue;
        };
        let diagnostics = outcome
            .issue
            .iter()
            .filter_map(|issue| issue.diagnostics.as_ref()?.value.clone())
            .collect::<Vec<_>>()
            .join(" ");
        report.warn(format!(
            "{label}: example {}/{} was not installed: {diagnostics}",
            key.0.as_ref(),
            key.1
        ));
        if let Some(count) = installed
            .get_mut(*owner)
            .and_then(|package| package.resources.as_mut())
            .and_then(|resources| resources.value.as_mut())
        {
            *count -= 1;
        }
    }
    Ok(())
}

async fn install<B: Backend>(
    context: ServerOperationContext<B>,
    tenant: TenantId,
    project: ProjectId,
    input: Install::Input,
) -> Result<Install::Output, OperationOutcomeError> {
    let config = &context.state.config.packages;
    let limits = Limits {
        max_uncompressed_bytes: config.max_uncompressed_bytes,
        max_files: config.max_files,
    };
    let dependencies = input
        .dependencies
        .and_then(|value| value.value)
        .unwrap_or(true);
    let examples = input
        .examples
        .and_then(|value| value.value)
        .unwrap_or(false);

    let mut roots = Vec::new();
    for upload in input.npmContent.unwrap_or_default() {
        let content = upload.value.unwrap_or_default();
        roots.push(Source::Uploaded(decode_upload(
            &content,
            config.max_package_bytes,
            &limits,
        )?));
    }
    for name in input.package.unwrap_or_default() {
        roots.push(Source::Named(name.value.unwrap_or_default().parse()?));
    }
    if roots.is_empty() {
        return Err(OperationOutcomeError::error(
            IssueType::required(),
            "Name a package to install ('package', for example 'hl7.fhir.us.core@6.1.0') \
             or upload one ('npmContent')."
                .to_string(),
        ));
    }

    let mut report = Report::default();
    let selected = select(
        &context,
        &tenant,
        &project,
        roots,
        dependencies,
        &mut report,
    )
    .await?;

    let canonicals = selected
        .iter()
        .filter_map(|selected| {
            let manifest = &selected.package.manifest;
            Some((manifest.name.clone(), manifest.canonical.clone()?))
        })
        .collect::<HashMap<_, _>>();

    let mut claimed = HashMap::new();
    let planned = selected
        .iter()
        .map(|selected| package_writes(selected, examples, &mut claimed, &mut report))
        .collect::<Vec<_>>();

    // What is in the project now: every id about to be written, and
    // everything the replaced versions installed.
    let keys = planned
        .iter()
        .flat_map(|(writes, _)| writes.iter().map(|write| write.key.clone()))
        .chain(
            selected
                .iter()
                .filter_map(|selected| selected.previous.as_ref())
                .flat_map(record_resources),
        )
        .collect::<Vec<_>>();
    let existing = read_existing(&context, &tenant, &project, keys).await?;

    let mut entries = Vec::new();
    // Examples go in a batch after the transaction (see `write_examples`),
    // each with the index of its package's report and its key.
    let mut example_entries = Vec::new();
    let mut example_owners = Vec::new();
    let mut installed = Vec::new();
    for (selected, (writes, own_guide)) in selected.into_iter().zip(planned) {
        let manifest = &selected.package.manifest;
        let label = format!("{}@{}", manifest.name, manifest.version);

        let mut written = Vec::new();
        for mut write in writes {
            if let Some(existing) = existing.get(&write.key)
                && let Err(reason) = may_replace(existing, &write, &manifest.name)
            {
                report.warn(format!(
                    "{label}: {}/{} was not installed, as {reason}.",
                    write.key.0.as_ref(),
                    write.key.1
                ));
                continue;
            }
            tag(&mut write.resource, manifest)?;
            let put = entry(HttpVerb::put(), &write.key, Some(write.resource));
            if write.example {
                example_entries.push(put);
                example_owners.push((installed.len(), label.clone(), write.key.clone()));
            } else {
                entries.push(put);
            }
            written.push((write.key, write.example));
        }

        // What the replaced version installed and this one does not.
        if let Some(previous) = &selected.previous {
            let kept = written.iter().map(|(key, _)| key).collect::<HashSet<_>>();
            for key in record_resources(previous) {
                if !kept.contains(&key)
                    && existing
                        .get(&key)
                        .is_some_and(|resource| installed_by(resource) == Some(&manifest.name))
                {
                    entries.push(entry(HttpVerb::delete(), &key, None));
                }
            }
        }

        let record = package_record(manifest, own_guide, &written, &canonicals);
        let record_key = (ResourceType::ImplementationGuide, record_id(&manifest.name));
        entries.push(entry(
            HttpVerb::put(),
            &record_key,
            Some(Resource::ImplementationGuide(record)),
        ));

        installed.push(Install::OutputPackage {
            name: *fhir_string(manifest.name.clone()),
            version: *fhir_string(manifest.version.clone()),
            status: FHIRCode {
                value: Some(
                    if selected.previous.is_some() {
                        "updated"
                    } else {
                        "installed"
                    }
                    .to_string(),
                ),
                ..Default::default()
            },
            previousVersion: selected
                .previous
                .as_ref()
                .and_then(version_of)
                .map(|version| *fhir_string(version)),
            resources: Some(FHIRInteger {
                value: Some(i64::try_from(written.len()).unwrap_or(i64::MAX)),
                ..Default::default()
            }),
            reason: None,
        });
    }

    if !entries.is_empty() {
        context
            .ctx
            .client
            .transaction(
                context.ctx.clone(),
                Bundle {
                    type_: BundleType::transaction(),
                    entry: Some(entries),
                    ..Default::default()
                },
            )
            .await?;
    }
    if !example_entries.is_empty() {
        write_examples(
            &context,
            example_entries,
            &example_owners,
            &mut installed,
            &mut report,
        )
        .await?;
    }
    // Last: validating an example resolves its profile, which before the
    // install is indexed finds the version it replaces.
    resolver::invalidate_project(&tenant, &project);

    if report.omitted_issues > 0 {
        let omitted = report.omitted_issues;
        report.issues.push(OperationOutcomeIssue {
            severity: IssueSeverity::warning(),
            code: IssueType::informational(),
            diagnostics: Some(fhir_string(format!("... and {omitted} more warnings."))),
            ..Default::default()
        });
    }

    installed.append(&mut report.packages);
    Ok(Install::Output {
        package: Some(installed).filter(|packages| !packages.is_empty()),
        outcome: if report.issues.is_empty() {
            None
        } else {
            Some(OperationOutcome {
                issue: report.issues,
                ..Default::default()
            })
        },
    })
}

async fn uninstall<B: Backend>(
    context: ServerOperationContext<B>,
    tenant: TenantId,
    project: ProjectId,
    input: Uninstall::Input,
) -> Result<Uninstall::Output, OperationOutcomeError> {
    let name = input.name.value.unwrap_or_default();
    let name = name.trim();
    let requested_version = input.version.and_then(|version| version.value);

    let record = installed_record(&context, &tenant, &project, name)
        .await?
        .ok_or_else(|| {
            OperationOutcomeError::error(
                IssueType::not_found(),
                format!("Package '{name}' is not installed in this project."),
            )
        })?;
    let installed_version = version_of(&record).unwrap_or_default();
    if let Some(requested) = requested_version
        && requested != installed_version
    {
        return Err(OperationOutcomeError::error(
            IssueType::conflict(),
            format!(
                "Version {installed_version} of '{name}' is installed, not {requested}; \
                 nothing was uninstalled."
            ),
        ));
    }

    let keys = record_resources(&record);
    let existing = read_existing(&context, &tenant, &project, keys.clone()).await?;
    let mut entries = keys
        .iter()
        .filter(|key| {
            existing
                .get(key)
                .is_some_and(|resource| installed_by(resource) == Some(name))
        })
        .map(|key| entry(HttpVerb::delete(), key, None))
        .collect::<Vec<_>>();
    let deleted = entries.len();
    entries.push(entry(
        HttpVerb::delete(),
        &(ResourceType::ImplementationGuide, record_id(name)),
        None,
    ));

    context
        .ctx
        .client
        .transaction(
            context.ctx.clone(),
            Bundle {
                type_: BundleType::transaction(),
                entry: Some(entries),
                ..Default::default()
            },
        )
        .await?;
    resolver::invalidate_project(&tenant, &project);

    Ok(Uninstall::Output {
        return_: OperationOutcome {
            issue: vec![OperationOutcomeIssue {
                severity: IssueSeverity::information(),
                code: IssueType::informational(),
                diagnostics: Some(fhir_string(format!(
                    "Uninstalled {name}@{installed_version}: deleted {deleted} resources."
                ))),
                ..Default::default()
            }],
            ..Default::default()
        },
    })
}

pub fn implementation_guide_install_op<B: Backend>()
-> OperationExecutor<ServerOperationContext<B>, Install::Input, Install::Output> {
    OperationExecutor::new(
        &Install::OPERATION_DEFINITION,
        Box::new(
            |context: ServerOperationContext<B>,
             tenant: TenantId,
             project: ProjectId,
             _request: &InvocationRequest,
             input: Install::Input| {
                Box::pin(install(context, tenant, project, input))
            },
        ),
    )
}

pub fn implementation_guide_uninstall_op<B: Backend>()
-> OperationExecutor<ServerOperationContext<B>, Uninstall::Input, Uninstall::Output> {
    OperationExecutor::new(
        &Uninstall::OPERATION_DEFINITION,
        Box::new(
            |context: ServerOperationContext<B>,
             tenant: TenantId,
             project: ProjectId,
             _request: &InvocationRequest,
             input: Uninstall::Input| {
                Box::pin(uninstall(context, tenant, project, input))
            },
        ),
    )
}
