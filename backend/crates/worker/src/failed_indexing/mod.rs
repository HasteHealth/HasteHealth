use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId, VersionId};
use haste_repository::types::FHIRMethod;

pub mod postgres;

/// Plain, storage-agnostic reference to a resource that failed search
/// indexing. Holds no `sqlx`/Postgres types so it can be shared by any
/// `FailedIndexingProvider` implementation, not just Postgres, and never
/// carries the resource body - just enough identity to look it back up.
#[derive(Clone, Debug)]
pub struct FailedIndexRecord {
    pub tenant: TenantId,
    pub project: ProjectId,
    pub version_id: VersionId,
    pub resource_type: String,
    pub fhir_method: FHIRMethod,
    pub error_message: String,
}

pub trait FailedIndexingProvider {
    /// Durably records resources that failed search indexing so they can be
    /// skipped (instead of retried forever) and inspected later. No-op on an
    /// empty list.
    fn record_failures(
        &self,
        failures: &[FailedIndexRecord],
    ) -> impl std::future::Future<Output = Result<(), OperationOutcomeError>> + Send;
}
