use haste_fhir_model::r4::{
    generated::resources::{Resource, ResourceType},
    generated::terminology::IssueType,
    sqlx::FHIRJson,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, ResourceId, TenantId};
use sqlx::PgConnection;

use crate::{
    pg::{PGConnection, StoreError},
    sequence::{ResourcePollingValue, ResourceSequential},
    types::FHIRMethod,
};

// 1. Concrete helper function accepting an explicit reference to remove HRTB issues entirely
async fn get_sequence_helper(
    executor: &mut PgConnection,
    tenant_id: &TenantId,
    cur_sequence: u64,
    count: Option<u64>,
) -> Result<Vec<ResourcePollingValue>, OperationOutcomeError> {
    let safe_sequence_row = sqlx::query_as::<_, (Option<i64>,)>(
        "SELECT max_safe_seq('resources_sequence_seq') as max_safe_seq",
    )
    .fetch_one(&mut *executor)
    .await
    .map_err(StoreError::from)?;

    let safe_sequence = safe_sequence_row.0.unwrap_or(0);

    let result = sqlx::query_as::<
        _,
        (
            String,
            TenantId,
            ProjectId,
            String,
            String,
            FHIRMethod,
            i64,
            FHIRJson<Resource>,
        ),
    >(
        r"
            SELECT id, tenant, project, version_id, resource_type, fhir_method, sequence, resource
            FROM resources
            WHERE tenant = $1 AND sequence > $2 AND sequence <= $3
            ORDER BY sequence
            LIMIT $4
        ",
    )
    .bind(tenant_id.as_ref())
    .bind(cur_sequence.cast_signed())
    .bind(safe_sequence)
    .bind(count.unwrap_or(100).cast_signed())
    .fetch_all(executor)
    .await
    .map_err(StoreError::from)?;

    result
        .into_iter()
        .map(
            |(
                id,
                tenant,
                project,
                version_id,
                resource_type_str,
                fhir_method,
                sequence,
                resource,
            )| {
                let resource_type = ResourceType::try_from(resource_type_str).map_err(|_| {
                    OperationOutcomeError::error(
                        IssueType::structure(),
                        "Invalid resource type encountered during sequence polling.".to_string(),
                    )
                })?;

                Ok::<ResourcePollingValue, OperationOutcomeError>(ResourcePollingValue {
                    id: ResourceId::new(id),
                    tenant,
                    project,
                    version_id,
                    resource_type,
                    fhir_method,
                    sequence, // Fixed: Left as i64 as expected by your model struct definition
                    resource,
                })
            },
        )
        .collect()
}

// 2. Trait implementation matching your PGConnection enum
impl ResourceSequential for PGConnection {
    async fn get_sequence(
        &self,
        tenant_id: &TenantId,
        sequence_id: u64,
        count: Option<u64>,
    ) -> Result<Vec<ResourcePollingValue>, OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => {
                // Acquire a dedicated connection from the pool so both queries execute
                // sequentially on the same PgConnection, matching the transaction path.
                let mut conn = pool.acquire().await.map_err(StoreError::from)?;
                get_sequence_helper(&mut conn, tenant_id, sequence_id, count).await
            }
            PGConnection::Transaction(tx, _) => {
                let mut conn = tx.lock().await;
                // Pass the mutable reference to the underlying PgConnection handle
                get_sequence_helper(&mut conn, tenant_id, sequence_id, count).await
            }
        }
    }
}
