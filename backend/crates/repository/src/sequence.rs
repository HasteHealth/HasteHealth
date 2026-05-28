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
    fn get_sequence(
        &self,
        tenant_id: &TenantId,
        sequence_id: u64,
        count: Option<u64>,
    ) -> impl Future<Output = Result<Vec<ResourcePollingValue>, OperationOutcomeError>> + Send;
}
