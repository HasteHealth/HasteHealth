//! Integration tests for the multi-row reads that decode each row by its
//! resource type: `read_latest`, `read_latest_many` and `history`.
//!
//! Requires `DATABASE_URL`, see `version_read.rs`.

use haste_fhir_client::{
    request::{FHIRHistoryTypeRequest, HistoryRequest},
    url::ParsedParameters,
};
use haste_fhir_model::r4::generated::resources::ResourceType;
use haste_jwt::{ProjectId, ResourceId, TenantId};
use haste_repository::{fhir::FHIRRepository, pg::PGConnection};
use sqlx::PgPool;

fn tenant() -> TenantId {
    TenantId::new("t".to_string()).unwrap()
}

fn project() -> ProjectId {
    ProjectId::new("p".to_string()).unwrap()
}

fn id(id: &str) -> ResourceId {
    ResourceId::new(id.to_string())
}

async fn setup(pool: &PgPool) -> sqlx::Result<()> {
    sqlx::query("INSERT INTO tenants (id) VALUES ('t')")
        .execute(pool)
        .await?;
    sqlx::query("INSERT INTO projects (tenant, id, fhir_version) VALUES ('t', 'p', 'r4')")
        .execute(pool)
        .await?;
    Ok(())
}

async fn store(
    pool: &PgPool,
    resource_type: &str,
    id: &str,
    version_id: &str,
    deleted: bool,
    request_method: &str,
) -> sqlx::Result<()> {
    let resource = format!(
        r#"{{"resourceType":"{resource_type}","id":"{id}","meta":{{"versionId":"{version_id}","lastUpdated":"2026-01-01T00:00:00Z"}}}}"#
    );
    sqlx::query(
        r"
            INSERT INTO resources
                (tenant, project, author_id, author_type, fhir_version, resource, request_method, fhir_method, deleted)
            VALUES ('t', 'p', 'test', 'Membership', 'r4', $1::jsonb, $2, 'update', $3)
        ",
    )
    .bind(resource)
    .bind(request_method)
    .bind(deleted)
    .execute(pool)
    .await?;
    Ok(())
}

/// A resource whose newest version is deleted is left out; a live one is returned.
#[sqlx::test(migrations = "./pg-migrations")]
async fn latest_reads_leave_out_deleted_resources(pool: PgPool) -> sqlx::Result<()> {
    setup(&pool).await?;
    store(&pool, "Patient", "live", "v1", false, "PUT").await?;
    store(&pool, "Patient", "gone", "v2", false, "PUT").await?;
    store(&pool, "Patient", "gone", "v3", true, "DELETE").await?;
    let repo = PGConnection::pool(pool);

    let read = |rid: &'static str| {
        let repo = &repo;
        async move {
            repo.read_latest(&tenant(), &project(), &ResourceType::Patient, &id(rid))
                .await
                .unwrap()
        }
    };
    assert!(read("live").await.is_some());
    assert!(read("gone").await.is_none());

    let keys = [
        (ResourceType::Patient, id("live")),
        (ResourceType::Patient, id("gone")),
    ];
    let many = repo
        .read_latest_many(&tenant(), &project(), &keys)
        .await
        .unwrap();
    assert_eq!(many.len(), 1);
    assert_eq!(many[0].id().as_deref(), Some("live"));
    Ok(())
}

/// History lists versions newest first, with the request method each was stored with.
#[sqlx::test(migrations = "./pg-migrations")]
async fn history_is_newest_first_with_request_methods(pool: PgPool) -> sqlx::Result<()> {
    setup(&pool).await?;
    store(&pool, "Patient", "h", "v1", false, "POST").await?;
    store(&pool, "Patient", "h", "v2", false, "PUT").await?;
    store(&pool, "Patient", "h", "v3", true, "DELETE").await?;
    let repo = PGConnection::pool(pool);

    let request = HistoryRequest::Type(FHIRHistoryTypeRequest {
        resource_type: ResourceType::Patient,
        parameters: ParsedParameters::new(vec![]),
    });
    let history = repo.history(&tenant(), &project(), &request).await.unwrap();

    let methods: Vec<&str> = history.iter().map(|v| v.request_method.as_str()).collect();
    assert_eq!(methods, ["DELETE", "PUT", "POST"]);
    Ok(())
}
