//! Writing rows to the `resources` table for `PGConnection`.
//!
//! A `PGConnection::Pool` write has nothing to buffer — it inserts
//! immediately via [`insert`], which only ever borrows the resource, so it
//! never clones or allocates for it. A `PGConnection::Transaction` write is
//! queued in [`PendingRows`] instead: earlier writes on the same transaction
//! must stay invisible to Postgres until flush/commit, and reads on that
//! transaction call [`PendingRows::flush`] first so they observe them. Since
//! a queued row has to outlive the call that created it, it needs its own
//! owned copy of the resource.
//!
use crate::{
    pg::StoreError,
    types::{FHIRMethod, SupportedFHIRVersions},
};
use haste_fhir_model::r4::{generated::resources::Resource, sqlx::FHIRJsonRef};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{AuthorId, AuthorKind, ProjectId, TenantId, claims::UserTokenClaims};
use sqlx::{PgExecutor, Postgres, QueryBuilder};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Common field access for anything that can be written as a `resources`
/// row, whether it owns its data ([`PendingResourceRow`]) or only borrows it
/// ([`BorrowedResourceRow`]). Lets `insert_batch` bind either kind without
/// caring which.
trait ResourceRowFields {
    fn tenant(&self) -> &TenantId;
    fn project(&self) -> &ProjectId;
    fn author_id(&self) -> &AuthorId;
    fn author_type(&self) -> &AuthorKind;
    fn fhir_version(&self) -> &SupportedFHIRVersions;
    fn resource(&self) -> &Resource;
    fn deleted(&self) -> bool;
    fn request_method(&self) -> &str;
    fn fhir_method(&self) -> &FHIRMethod;
}

/// A single buffered `resources` row awaiting a batched multi-row INSERT.
#[derive(Debug, Clone)]
struct PendingResourceRow {
    tenant: TenantId,
    project: ProjectId,
    author_id: AuthorId,
    author_type: AuthorKind,
    fhir_version: SupportedFHIRVersions,
    resource: Resource,
    deleted: bool,
    request_method: &'static str,
    fhir_method: FHIRMethod,
}

impl ResourceRowFields for PendingResourceRow {
    fn tenant(&self) -> &TenantId {
        &self.tenant
    }
    fn project(&self) -> &ProjectId {
        &self.project
    }
    fn author_id(&self) -> &AuthorId {
        &self.author_id
    }
    fn author_type(&self) -> &AuthorKind {
        &self.author_type
    }
    fn fhir_version(&self) -> &SupportedFHIRVersions {
        &self.fhir_version
    }
    fn resource(&self) -> &Resource {
        &self.resource
    }
    fn deleted(&self) -> bool {
        self.deleted
    }
    fn request_method(&self) -> &str {
        self.request_method
    }
    fn fhir_method(&self) -> &FHIRMethod {
        &self.fhir_method
    }
}

/// A `resources` row for an immediate, unbuffered INSERT — every field is
/// borrowed straight from the caller, so writing it costs no clone and no
/// allocation beyond the query itself.
struct BorrowedResourceRow<'a> {
    tenant: &'a TenantId,
    project: &'a ProjectId,
    author_id: &'a AuthorId,
    author_type: &'a AuthorKind,
    fhir_version: &'a SupportedFHIRVersions,
    resource: &'a Resource,
    deleted: bool,
    request_method: &'static str,
    fhir_method: FHIRMethod,
}

impl ResourceRowFields for BorrowedResourceRow<'_> {
    fn tenant(&self) -> &TenantId {
        self.tenant
    }
    fn project(&self) -> &ProjectId {
        self.project
    }
    fn author_id(&self) -> &AuthorId {
        self.author_id
    }
    fn author_type(&self) -> &AuthorKind {
        self.author_type
    }
    fn fhir_version(&self) -> &SupportedFHIRVersions {
        self.fhir_version
    }
    fn resource(&self) -> &Resource {
        self.resource
    }
    fn deleted(&self) -> bool {
        self.deleted
    }
    fn request_method(&self) -> &str {
        self.request_method
    }
    fn fhir_method(&self) -> &FHIRMethod {
        &self.fhir_method
    }
}

/// Inserts a single `resources` row immediately. Used for
/// `PGConnection::Pool` writes, which are never buffered and so never need
/// to own the resource.
#[allow(clippy::too_many_arguments)]
pub async fn execute<'e, E>(
    executor: E,
    tenant: &TenantId,
    project: &ProjectId,
    author: &UserTokenClaims,
    fhir_version: &SupportedFHIRVersions,
    resource: &Resource,
    deleted: bool,
    request_method: &'static str,
    fhir_method: FHIRMethod,
) -> Result<(), OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let row = BorrowedResourceRow {
        tenant,
        project,
        author_id: &author.sub,
        author_type: &author.resource_type,
        fhir_version,
        resource,
        deleted,
        request_method,
        fhir_method,
    };

    insert_resource_updates(executor, std::slice::from_ref(&row)).await
}

/// Rows queued on an open transaction for one batched multi-row INSERT at
/// flush/commit time, instead of one INSERT per write. Cheap to clone — it
/// shares the same underlying buffer, which matters when a nested
/// `transaction()` call reuses its parent's queue.
#[derive(Debug, Clone, Default)]
pub struct PendingRows(Arc<Mutex<Vec<PendingResourceRow>>>);

impl PendingRows {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Buffers a row. Unlike [`insert`], `resource` must be an owned copy —
    /// this data has to outlive the call that queued it, until `flush` runs.
    #[allow(clippy::too_many_arguments)]
    pub async fn push(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        author: &UserTokenClaims,
        fhir_version: &SupportedFHIRVersions,
        resource: Resource,
        deleted: bool,
        request_method: &'static str,
        fhir_method: FHIRMethod,
    ) {
        self.0.lock().await.push(PendingResourceRow {
            tenant: tenant.clone(),
            project: project.clone(),
            author_id: author.sub.clone(),
            author_type: author.resource_type.clone(),
            fhir_version: fhir_version.clone(),
            resource,
            deleted,
            request_method,
            fhir_method,
        });
    }

    /// Drains and inserts every buffered row, so a subsequent read on the
    /// same transaction observes prior writes that haven't reached Postgres
    /// yet. The internal lock is released before tx is locked, so the two
    /// are never held simultaneously.
    ///
    /// # Errors
    ///
    /// Returns an [`OperationOutcomeError`] if inserting the buffered rows
    /// into Postgres fails.
    pub async fn flush(
        &self,
        tx: &Arc<Mutex<sqlx::Transaction<'static, Postgres>>>,
    ) -> Result<(), OperationOutcomeError> {
        let rows = {
            let mut guard = self.0.lock().await;
            std::mem::take(&mut *guard)
        };

        if rows.is_empty() {
            return Ok(());
        }

        let mut conn = tx.lock().await;
        insert_resource_updates(&mut **conn, &rows).await
    }
}

/// Executes one multi-row INSERT for every row given. No-op on an empty
/// slice (`QueryBuilder::push_values` panics if given zero tuples).
async fn insert_resource_updates<'e, E, R>(
    executor: E,
    rows: &[R],
) -> Result<(), OperationOutcomeError>
where
    E: PgExecutor<'e>,
    R: ResourceRowFields,
{
    if rows.is_empty() {
        return Ok(());
    }

    let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
        "INSERT INTO resources (tenant, project, author_id, fhir_version, resource, deleted, request_method, author_type, fhir_method) ",
    );

    query_builder.push_values(rows, |mut b, row| {
        b.push_bind(row.tenant().as_ref())
            .push_bind(row.project().as_ref())
            .push_bind(row.author_id().as_ref())
            .push_bind(row.fhir_version())
            .push_bind(FHIRJsonRef(row.resource()))
            .push_bind(row.deleted())
            .push_bind(row.request_method())
            .push_bind(row.author_type().as_ref())
            .push_bind(row.fhir_method());
    });

    query_builder
        .build()
        .execute(executor)
        .await
        .map_err(StoreError::from)?;

    Ok(())
}
