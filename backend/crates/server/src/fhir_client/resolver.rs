use crate::fhir_client::ServerCTX;
use haste_fhir_client::{FHIRClient, canonical_resolver::CanonicalResolver};
use haste_fhir_model::r4::generated::{
    resources::{Resource, ResourceType},
    types::FHIRString,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId};
use haste_reflect::MetaValue;
use moka::future::{Cache, CacheBuilder};
use std::sync::{Arc, LazyLock};

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

/// Resolved canonicals. Only hits are kept: a canonical that is missing now
/// is looked up again next time, so a profile resolves as soon as it is
/// created and indexed (an implementation guide installed, say).
///
/// A lookup reads what the search index has, so one made after a canonical
/// resource is updated but before the update is indexed finds the previous
/// version. Entries therefore live at most five minutes, which also bounds
/// how long another server process serves a version this one replaced.
static CACHE: LazyLock<Cache<String, Arc<Resource>>> = LazyLock::new(|| {
    CacheBuilder::new(10_000)
        .time_to_live(std::time::Duration::from_mins(5))
        .support_invalidation_closures()
        .build()
});

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
        if let Some(cached) = CACHE.get(&key).await {
            return Ok(Some(cached));
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

        if let Some(resolved) = &resolved {
            CACHE.insert(key, resolved.clone()).await;
        }
        Ok(resolved)
    }
}
