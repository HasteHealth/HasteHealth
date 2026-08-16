use crate::types::FHIRMethod;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId, VersionId};
use sqlx::types::time::OffsetDateTime;

/// Plain, storage-agnostic reference to a resource that failed search
/// indexing. Never carries the resource body - just enough identity to look
/// it back up.
#[derive(Clone, Debug)]
pub struct FailedIndexRecord {
    pub tenant: TenantId,
    pub project: ProjectId,
    pub version_id: VersionId,
    pub resource_type: String,
    pub fhir_method: FHIRMethod,
    pub error_message: String,
}

/// A previously recorded indexing failure, as read back for display -
/// includes the bookkeeping columns (`attempt_count`, timestamps) that only
/// exist once a `FailedIndexRecord` has actually been persisted.
#[derive(sqlx::FromRow, Clone, Debug)]
pub struct FailedIndexEntry {
    pub id: String,
    pub version_id: String,
    pub resource_type: String,
    pub fhir_method: FHIRMethod,
    /// Current sequence position of this version in the `resources` table -
    /// looked up at read time since it isn't part of the failure record itself.
    pub sequence: i64,
    pub attempt_count: i32,
    pub error_message: String,
    pub first_failed_at: OffsetDateTime,
    pub last_failed_at: OffsetDateTime,
    pub resolved_at: Option<OffsetDateTime>,
}

pub trait FailedIndexingProvider {
    /// Durably records resources that failed search indexing so they can be
    /// skipped (instead of retried forever) and inspected later. No-op on an
    /// empty list.
    fn record_failures(
        &self,
        failures: &[FailedIndexRecord],
    ) -> impl std::future::Future<Output = Result<(), OperationOutcomeError>> + Send;

    /// Lists recorded indexing failures for a tenant/project, most recently
    /// failed first.
    fn search(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
    ) -> impl std::future::Future<Output = Result<Vec<FailedIndexEntry>, OperationOutcomeError>> + Send;
}
