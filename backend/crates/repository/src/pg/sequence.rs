use haste_fhir_model::r4::sqlx::FHIRJson;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, ResourceId, TenantId};
use sqlx::{PgConnection, Row, postgres::PgRow};

use crate::{
    pg::{PGConnection, StoreError, decode_resource, row_resource_type},
    sequence::{ResourcePollingValue, ResourceSequential},
};

/// The watermark half of a sequence poll; see
/// [`ResourceSequential::max_safe_sequence`]. Its own statement on purpose:
/// `max_safe_seq` reads the sequence and scans `pg_locks` inside the
/// function, and a row select that uses the result must take its snapshot
/// after that scan, which a later statement under READ COMMITTED does.
async fn max_safe_sequence_helper(
    executor: &mut PgConnection,
) -> Result<i64, OperationOutcomeError> {
    let safe_sequence: Option<i64> =
        sqlx::query_scalar("SELECT max_safe_seq('resources_sequence_seq')")
            .fetch_one(&mut *executor)
            .await
            .map_err(StoreError::from)?;

    Ok(safe_sequence.unwrap_or(0))
}

// 1. Concrete helper function accepting an explicit reference to remove HRTB issues entirely
async fn get_sequence_helper(
    executor: &mut PgConnection,
    tenant_id: &TenantId,
    project_id: &ProjectId,
    cur_sequence: u64,
    safe_sequence: i64,
    count: Option<u64>,
) -> Result<Vec<ResourcePollingValue>, OperationOutcomeError> {
    sqlx::query(
        r"
            SELECT id, tenant, project, version_id, resource_type, fhir_method, sequence, resource
            FROM resources
            WHERE tenant = $1 AND project = $2 AND sequence > $3 AND sequence <= $4
            ORDER BY sequence
            LIMIT $5
        ",
    )
    .bind(tenant_id.as_ref())
    .bind(project_id.as_ref())
    .bind(cur_sequence.cast_signed())
    .bind(safe_sequence)
    .bind(count.unwrap_or(100).cast_signed())
    .try_map(|row: PgRow| {
        let resource_type = row_resource_type(&row)?;
        Ok(ResourcePollingValue {
            id: ResourceId::new(row.try_get::<String, _>("id")?),
            tenant: row.try_get("tenant")?,
            project: row.try_get("project")?,
            version_id: row.try_get("version_id")?,
            fhir_method: row.try_get("fhir_method")?,
            sequence: row.try_get("sequence")?,
            resource: FHIRJson(decode_resource(&row, resource_type)?),
            resource_type,
        })
    })
    .fetch_all(executor)
    .await
    .map_err(|e| StoreError::from(e).into())
}

// 2. Trait implementation matching your PGConnection enum
impl ResourceSequential for PGConnection {
    async fn max_safe_sequence(&self) -> Result<i64, OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => {
                let mut conn = pool.acquire().await.map_err(StoreError::from)?;
                max_safe_sequence_helper(&mut conn).await
            }
            PGConnection::Transaction(tx, _, _) => {
                let mut conn = tx.lock().await;
                max_safe_sequence_helper(&mut conn).await
            }
        }
    }

    async fn get_sequence(
        &self,
        tenant_id: &TenantId,
        project_id: &ProjectId,
        sequence_id: u64,
        safe_sequence: i64,
        count: Option<u64>,
    ) -> Result<Vec<ResourcePollingValue>, OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => {
                let mut conn = pool.acquire().await.map_err(StoreError::from)?;
                get_sequence_helper(
                    &mut conn,
                    tenant_id,
                    project_id,
                    sequence_id,
                    safe_sequence,
                    count,
                )
                .await
            }
            PGConnection::Transaction(tx, _, _) => {
                let mut conn = tx.lock().await;
                // Pass the mutable reference to the underlying PgConnection handle
                get_sequence_helper(
                    &mut conn,
                    tenant_id,
                    project_id,
                    sequence_id,
                    safe_sequence,
                    count,
                )
                .await
            }
        }
    }
}
