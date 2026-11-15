//! The cache of resource versions read by version id.

use haste_fhir_model::r4::generated::resources::Resource;
use haste_jwt::{ProjectId, TenantId, VersionId};
use moka::future::Cache;
use std::sync::Arc;

/// A cached version and the tenant and project it was read for.
#[derive(Debug)]
struct Entry {
    tenant: TenantId,
    project: ProjectId,
    resource: Resource,
}

/// Resource versions by version id. A version belongs to the tenant and
/// project it was read for and is answered to no other, so a lookup here
/// finds only what the same lookup in the database would.
///
/// The key is the version id alone, which a lookup borrows as it is; the
/// tenant and project are compared on the entry found.
#[derive(Debug, Clone)]
pub struct VersionCache(Cache<VersionId, Arc<Entry>>);

impl VersionCache {
    #[must_use]
    pub fn new(max_capacity: u64) -> Self {
        Self(Cache::new(max_capacity))
    }

    /// The version `version_id`, when it is cached for `tenant` and `project`.
    pub async fn get(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        version_id: &VersionId,
    ) -> Option<Resource> {
        let entry = self.0.get(version_id).await?;

        (entry.tenant == *tenant && entry.project == *project).then(|| entry.resource.clone())
    }

    /// Caches `resource` as the version `version_id` of `tenant` and `project`.
    pub async fn insert(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        version_id: VersionId,
        resource: Resource,
    ) {
        let entry = Entry {
            tenant: tenant.clone(),
            project: project.clone(),
            resource,
        };
        self.0.insert(version_id, Arc::new(entry)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use haste_fhir_model::r4::generated::resources::ResourceType;

    fn tenant(id: &str) -> TenantId {
        TenantId::new(id.to_string()).unwrap()
    }

    fn project(id: &str) -> ProjectId {
        ProjectId::new(id.to_string()).unwrap()
    }

    fn patient(id: &str) -> Resource {
        let json = format!(r#"{{"resourceType":"Patient","id":"{id}"}}"#);
        ResourceType::Patient.deserialize(json.as_bytes()).unwrap()
    }

    async fn cached() -> (VersionCache, VersionId) {
        let cache = VersionCache::new(10);
        let version_id = VersionId::new("version-1".to_string());
        cache
            .insert(
                &tenant("tenant-a"),
                &project("project-a"),
                version_id.clone(),
                patient("p1"),
            )
            .await;

        (cache, version_id)
    }

    #[tokio::test]
    async fn a_version_is_found_for_its_tenant_and_project() {
        let (cache, version_id) = cached().await;

        let found = cache
            .get(&tenant("tenant-a"), &project("project-a"), &version_id)
            .await;

        assert_eq!(found.unwrap().id().as_deref(), Some("p1"));
    }

    #[tokio::test]
    async fn a_version_is_not_found_for_another_tenant() {
        let (cache, version_id) = cached().await;

        let found = cache
            .get(&tenant("tenant-b"), &project("project-a"), &version_id)
            .await;

        assert!(found.is_none());
    }

    #[tokio::test]
    async fn a_version_is_not_found_for_another_project() {
        let (cache, version_id) = cached().await;

        let found = cache
            .get(&tenant("tenant-a"), &project("project-b"), &version_id)
            .await;

        assert!(found.is_none());
    }
}
