//! Integration tests for reading resource versions: `read_version` and the
//! cache behind `read_by_version_ids`.
//!
//! Requires a reachable Postgres instance, e.g. `docker-services-compose.yml`'s
//! `postgres` service, with `DATABASE_URL` set to an admin-capable connection
//! string (`sqlx::test` creates and drops an ephemeral database per test):
//!
//! ```sh
//! DATABASE_URL=postgresql://postgres:postgres@127.0.0.1/haste_health cargo test -p haste-repository
//! ```

use haste_fhir_model::r4::generated::resources::{Resource, ResourceType};
use haste_jwt::{ProjectId, ResourceId, TenantId, VersionId};
use haste_repository::{
    fhir::{CachePolicy, FHIRRepository},
    pg::PGConnection,
};
use sqlx::PgPool;

fn tenant(id: &str) -> TenantId {
    TenantId::new(id.to_string()).unwrap()
}

fn project(id: &str) -> ProjectId {
    ProjectId::new(id.to_string()).unwrap()
}

fn version(id: &str) -> VersionId {
    VersionId::new(id.to_string())
}

fn id(id: &str) -> ResourceId {
    ResourceId::new(id.to_string())
}

async fn create_project(pool: &PgPool, tenant: &str, project: &str) -> sqlx::Result<()> {
    sqlx::query("INSERT INTO tenants (id) VALUES ($1) ON CONFLICT DO NOTHING")
        .bind(tenant)
        .execute(pool)
        .await?;
    sqlx::query("INSERT INTO projects (tenant, id, fhir_version) VALUES ($1, $2, 'r4')")
        .bind(tenant)
        .bind(project)
        .execute(pool)
        .await?;
    Ok(())
}

/// Stores the version `version_id` of `resource_type/id` in `tenant` and
/// `project`.
async fn store_version(
    pool: &PgPool,
    tenant: &str,
    project: &str,
    resource_type: &str,
    id: &str,
    version_id: &str,
) -> sqlx::Result<()> {
    let resource = format!(
        r#"{{"resourceType":"{resource_type}","id":"{id}","meta":{{"versionId":"{version_id}","lastUpdated":"2026-01-01T00:00:00Z"}}}}"#
    );
    sqlx::query(
        r"
            INSERT INTO resources
                (tenant, project, author_id, author_type, fhir_version, resource, request_method, fhir_method)
            VALUES ($1, $2, 'test', 'Membership', 'r4', $3::jsonb, 'POST', 'create')
        ",
    )
    .bind(tenant)
    .bind(project)
    .bind(resource)
    .execute(pool)
    .await?;
    Ok(())
}

fn identity(resource: &Resource) -> (ResourceType, Option<String>) {
    (resource.resource_type(), resource.id().clone())
}

/// A version is read as a version of one resource: its type and id have to
/// be that resource's, in the tenant and project it is stored in.
#[sqlx::test(migrations = "./pg-migrations")]
async fn a_version_is_read_only_as_its_own_resource(pool: PgPool) -> sqlx::Result<()> {
    create_project(&pool, "tenant-a", "project-a").await?;
    create_project(&pool, "tenant-a", "project-b").await?;
    create_project(&pool, "tenant-b", "project-a").await?;
    store_version(&pool, "tenant-a", "project-a", "Patient", "p1", "v-p1").await?;
    let repo = PGConnection::pool(pool);

    let read = |tenant_id: &'static str,
                project_id: &'static str,
                resource_type: ResourceType,
                resource_id: &'static str,
                version_id: &'static str| {
        let repo = &repo;
        async move {
            repo.read_version(
                &tenant(tenant_id),
                &project(project_id),
                &resource_type,
                &id(resource_id),
                &version(version_id),
            )
            .await
            .unwrap()
        }
    };

    let found = read("tenant-a", "project-a", ResourceType::Patient, "p1", "v-p1").await;
    assert_eq!(
        found.as_ref().map(identity),
        Some((ResourceType::Patient, Some("p1".to_string())))
    );

    let another_type = read(
        "tenant-a",
        "project-a",
        ResourceType::Observation,
        "p1",
        "v-p1",
    );
    assert!(another_type.await.is_none());
    let another_id = read("tenant-a", "project-a", ResourceType::Patient, "p2", "v-p1");
    assert!(another_id.await.is_none());
    let another_version = read("tenant-a", "project-a", ResourceType::Patient, "p1", "v-0");
    assert!(another_version.await.is_none());
    let another_project = read("tenant-a", "project-b", ResourceType::Patient, "p1", "v-p1");
    assert!(another_project.await.is_none());
    let another_tenant = read("tenant-b", "project-a", ResourceType::Patient, "p1", "v-p1");
    assert!(another_tenant.await.is_none());

    Ok(())
}

/// A version cached for one tenant and project is found there again, and
/// nowhere else.
#[sqlx::test(migrations = "./pg-migrations")]
async fn a_cached_version_stays_in_its_tenant_and_project(pool: PgPool) -> sqlx::Result<()> {
    create_project(&pool, "tenant-a", "project-a").await?;
    create_project(&pool, "tenant-a", "project-b").await?;
    create_project(&pool, "tenant-b", "project-a").await?;
    store_version(&pool, "tenant-a", "project-a", "Patient", "p1", "v-p1").await?;
    let repo = PGConnection::pool(pool);
    let version_id = version("v-p1");

    let read = |tenant_id: &'static str, project_id: &'static str, cache_policy: CachePolicy| {
        let (repo, version_id) = (&repo, &version_id);
        async move {
            repo.read_by_version_ids(
                &tenant(tenant_id),
                &project(project_id),
                &[version_id],
                cache_policy,
            )
            .await
            .unwrap()
        }
    };

    // The first read caches the version, the second is answered from the cache.
    for _ in 0..2 {
        let found = read("tenant-a", "project-a", CachePolicy::Cache).await;
        assert_eq!(found.len(), 1);
    }

    for cache_policy in [CachePolicy::Cache, CachePolicy::NoCache] {
        let another_tenant = read("tenant-b", "project-a", cache_policy).await;
        assert!(another_tenant.is_empty());
    }
    for cache_policy in [CachePolicy::Cache, CachePolicy::NoCache] {
        let another_project = read("tenant-a", "project-b", cache_policy).await;
        assert!(another_project.is_empty());
    }

    Ok(())
}
