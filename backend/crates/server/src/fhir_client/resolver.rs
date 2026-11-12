use crate::fhir_client::ServerCTX;
use haste_fhir_client::{FHIRClient, canonical_resolver::CanonicalResolver};
use haste_fhir_model::r4::generated::{
    resources::{Resource, ResourceType},
    types::FHIRString,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId};
use haste_reflect::MetaValue;
use moka::{
    Expiry,
    future::{Cache, CacheBuilder},
};
use std::{
    sync::{Arc, LazyLock},
    time::{Duration, Instant},
};

fn generate_key(
    tenant_id: &TenantId,
    project_id: &ProjectId,
    resource_type: &ResourceType,
    url: &str,
) -> String {
    format!(
        "{}::{}::{}::{}",
        tenant_id,
        project_id,
        resource_type.as_ref(),
        url
    )
}

/// What a lookup found: the resource, or that the canonical is unknown.
#[derive(Clone)]
enum Cached {
    Hit(Arc<Resource>),
    Miss,
}

/// How long a resolved canonical is served from the cache.
///
/// A lookup reads what the search index has, so one made after a canonical
/// resource is updated but before the update is indexed finds the previous
/// version. Entries therefore live at most five minutes, which also bounds
/// how long another server process serves a version this one replaced.
const HIT_TTL: Duration = Duration::from_mins(5);

/// How long an unknown canonical is remembered before it is searched again.
///
/// A resource whose `meta.profile` names a profile the project does not have
/// would otherwise run a `StructureDefinition` search on every write. The entry
/// is short so that a profile created on another server process resolves
/// soon after it is indexed; on the process that stores the profile the entry
/// is dropped at once (see `invalidate_misses`).
const MISS_TTL: Duration = Duration::from_secs(30);

struct CanonicalExpiry;

impl Expiry<String, Cached> for CanonicalExpiry {
    fn expire_after_create(
        &self,
        _key: &String,
        value: &Cached,
        _created_at: Instant,
    ) -> Option<Duration> {
        Some(match value {
            Cached::Hit(_) => HIT_TTL,
            Cached::Miss => MISS_TTL,
        })
    }
}

/// Resolved canonicals, hits and misses with their own lifetimes.
static CACHE: LazyLock<Cache<String, Cached>> = LazyLock::new(|| {
    CacheBuilder::new(10_000)
        .expire_after(CanonicalExpiry)
        .support_invalidation_closures()
        .build()
});

/// Forgets that canonicals were missing from the project, after one of its
/// canonical resources (a `StructureDefinition`, `ValueSet` or `CodeSystem`) is
/// written, so a profile resolves on this process as soon as it is indexed.
pub fn invalidate_misses(tenant_id: &TenantId, project_id: &ProjectId) {
    let prefix = format!("{tenant_id}::{project_id}::");
    if let Err(error) = CACHE.invalidate_entries_if(move |key, value| {
        matches!(value, Cached::Miss) && key.starts_with(&prefix)
    }) {
        tracing::warn!("Failed to invalidate the canonical resolver cache: {error}");
    }
}

/// Forgets this process's resolutions for the project, after its artifacts
/// change in bulk (an implementation guide installed or uninstalled), so the
/// next lookup sees the new versions.
pub fn invalidate_project(tenant_id: &TenantId, project_id: &ProjectId) {
    let prefix = format!("{tenant_id}::{project_id}::");
    if let Err(error) = CACHE.invalidate_entries_if(move |key, _| key.starts_with(&prefix)) {
        tracing::warn!("Failed to invalidate the canonical resolver cache: {error}");
    }
}

/// The version of a canonical resource.
fn version(resource: &Resource) -> Option<&str> {
    resource
        .get_field("version")
        .and_then(|version| version.as_any().downcast_ref::<FHIRString>())
        .and_then(|version| version.value.as_deref())
}

pub struct ServerCTXResolver<Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError>>(
    Arc<ServerCTX<Client>>,
);

impl<Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError>> ServerCTXResolver<Client> {
    pub fn new(ctx: Arc<ServerCTX<Client>>) -> Self {
        Self(ctx)
    }
}

impl<Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError>> Clone
    for ServerCTXResolver<Client>
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError>> CanonicalResolver
    for ServerCTXResolver<Client>
{
    async fn resolve(
        &self,
        resource_type: ResourceType,
        canonical_url: &str,
    ) -> Result<Option<Arc<Resource>>, OperationOutcomeError> {
        let key = generate_key(
            &self.0.tenant,
            &self.0.project,
            &resource_type,
            canonical_url,
        );
        match CACHE.get(&key).await {
            Some(Cached::Hit(cached)) => return Ok(Some(cached)),
            Some(Cached::Miss) => return Ok(None),
            None => {}
        }

        let mut parts = canonical_url.splitn(2, '|');
        let url = parts.next().unwrap_or_default();
        let requested_version = parts.next().filter(|version| !version.is_empty());

        // The project's own resources come before the system's in the search
        // (see `SetArtifactTenant`), so a project's copy of a canonical takes
        // precedence over the built-in one, as a read by id does.
        let candidates = self
            .0
            .client
            .search_type(
                self.0.clone(),
                resource_type,
                vec![("url".to_string(), vec![url.to_string()])].into(),
            )
            .await?
            .entry
            .unwrap_or_default()
            .into_iter()
            .filter_map(|entry| entry.resource)
            .collect::<Vec<_>>();

        // A requested version is preferred, but a lookup without a match
        // still resolves to the canonical's other version, as it always has.
        let index = requested_version
            .and_then(|requested| {
                candidates
                    .iter()
                    .position(|resource| version(resource) == Some(requested))
            })
            .unwrap_or(0);
        let resolved = candidates
            .into_iter()
            .nth(index)
            .map(|resource| Arc::new(*resource));

        let cached = resolved.clone().map_or(Cached::Miss, Cached::Hit);
        CACHE.insert(key, cached).await;
        Ok(resolved)
    }
}
