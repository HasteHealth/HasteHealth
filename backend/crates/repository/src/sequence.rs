use haste_fhir_model::r4::{
    generated::resources::{Resource, ResourceType},
    sqlx::FHIRJson,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, ResourceId, TenantId};

use crate::types::FHIRMethod;

#[derive(Clone)]
pub struct ResourcePollingValue {
    pub id: ResourceId,
    pub resource_type: ResourceType,
    pub version_id: String,
    pub project: ProjectId,
    pub tenant: TenantId,
    pub resource: FHIRJson<Resource>,
    pub sequence: i64,
    pub fhir_method: FHIRMethod,
}

pub trait ResourceSequential {
    /// The highest `resources.sequence` a consumer may read up to without
    /// skipping a row that is still being written: every row at or below it
    /// that will ever commit is visible to any snapshot taken after this call
    /// returns.
    fn max_safe_sequence(&self) -> impl Future<Output = Result<i64, OperationOutcomeError>> + Send;

    /// The project's rows with `sequence_id < sequence <= safe_sequence`, in
    /// sequence order. `safe_sequence` must come from a
    /// [`max_safe_sequence`](Self::max_safe_sequence) call that returned
    /// before this query's snapshot was taken.
    fn get_sequence(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        sequence_id: u64,
        safe_sequence: i64,
        count: Option<u64>,
    ) -> impl Future<Output = Result<Vec<ResourcePollingValue>, OperationOutcomeError>> + Send;
}
