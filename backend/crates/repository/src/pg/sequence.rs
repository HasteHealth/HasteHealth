use haste_fhir_model::r4::{
    generated::resources::{Resource, ResourceType},
    sqlx::FHIRJson,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, ResourceId, TenantId};
use sqlx::{Acquire, Postgres};

use crate::{
    pg::{PGConnection, StoreError},
    sequence::{ResourcePollingValue, ResourceSequential},
    types::FHIRMethod,
};

fn get_sequence<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant_id: &'a TenantId,
    cur_sequence: u64,
    count: Option<u64>,
) -> impl Future<Output = Result<Vec<ResourcePollingValue>, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::from)?;
        // Run as a transaction to ensure safe sequence retrieval.
        // Run as seperate query.
        // Isolation level must be set to allowe dirty reads from pg_locks.
        // This is to ensure that we can read the safe sequence even if other transactions are in progress.
        let safe_sequence =
            sqlx::query!("SELECT max_safe_seq('resources_sequence_seq') as max_safe_seq")
                .fetch_one(&mut *conn)
                .await
                .map_err(StoreError::from)?
                .max_safe_seq
                .unwrap_or(0);

        let result = sqlx::query_as!(
            ResourcePollingValue,
            r#"SELECT  id as "id: ResourceId", 
                       tenant as "tenant: TenantId", 
                       project as "project: ProjectId", 
                       version_id, 
                       resource_type as "resource_type: ResourceType", 
                       fhir_method as "fhir_method: FHIRMethod", 
                       sequence, 
                       resource as "resource: FHIRJson<Resource>"
            FROM resources WHERE tenant = $1 AND sequence > $2 AND sequence <= $3 ORDER BY sequence LIMIT $4 "#,
            tenant_id.as_ref() as &str,
            cur_sequence as i64,
            safe_sequence,
            count.unwrap_or(100) as i64
        )
        .fetch_all(&mut *conn)
        .await
        .map_err(StoreError::from)?;

        // if !result.is_empty() {
        //     println!("safe_sequence: {:?}", safe_sequence);
        // }

        Ok(result)
    }
}

impl ResourceSequential for PGConnection {
    async fn get_sequence(
        &self,
        tenant_id: &TenantId,
        sequence_id: u64,
        count: Option<u64>,
    ) -> Result<Vec<ResourcePollingValue>, OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => {
                let res = get_sequence(pool, tenant_id, sequence_id, count).await?;
                Ok(res)
            }
            PGConnection::Transaction(tx, _) => {
                let mut conn = tx.lock().await;

                // Handle PgConnection connection
                let res = get_sequence(&mut *conn, tenant_id, sequence_id, count).await?;
                Ok(res)
            }
        }
    }
}
