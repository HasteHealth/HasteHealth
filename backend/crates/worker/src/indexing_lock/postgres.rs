use crate::indexing_lock::IndexLockProvider;
use haste_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use haste_jwt::TenantId;
use haste_repository::pg::PGConnection;
use sqlx::{Acquire, Postgres, QueryBuilder};

#[derive(OperationOutcomeError, Debug)]
pub enum TenantLockIndexError {
    #[fatal(code = "exception", diagnostic = "SQL error occurred {arg0}")]
    SQLError(#[from] sqlx::Error),
    #[fatal(
        code = "exception",
        diagnostic = "Locking must be done in a transaction."
    )]
    InvalidConnection,
}

#[derive(sqlx::FromRow, Debug)]
pub struct TenantLockIndex {
    #[allow(dead_code)]
    pub id: TenantId,
    pub index_sequence_position: i64,
}

impl IndexLockProvider<TenantId, TenantLockIndex> for PGConnection {
    async fn get_available_locks(
        &self,
        tenants: Vec<&TenantId>,
    ) -> Result<Vec<TenantLockIndex>, OperationOutcomeError> {
        match self {
            PGConnection::Transaction(tx, _) => {
                let mut tx = tx.lock().await;
                let conn = (&mut (*tx))
                    .acquire()
                    .await
                    .map_err(TenantLockIndexError::from)?;
                // Implementation for retrieving available locks from PostgreSQL

                let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
                    "SELECT id, index_sequence_position FROM tenants WHERE id IN ( ",
                );

                let mut separated = query_builder.separated(", ");
                for tenant_id in tenants.iter() {
                    separated.push_bind(tenant_id.as_ref());
                }

                separated.push_unseparated(") FOR NO KEY UPDATE SKIP LOCKED");

                let query = query_builder.build_query_as();
                // println!("Executing query: '{:?}'", query.sql());
                let res = query
                    .fetch_all(conn)
                    .await
                    .map_err(TenantLockIndexError::from)?;

                Ok(res)
            }
            _ => Err(TenantLockIndexError::InvalidConnection.into()),
        }
    }

    async fn update_lock(
        &self,
        tenant_id: &TenantId,
        model: TenantLockIndex,
    ) -> Result<(), OperationOutcomeError> {
        match self {
            PGConnection::Transaction(tx, _) => {
                let mut tx = tx.lock().await;
                let conn = (&mut (*tx))
                    .acquire()
                    .await
                    .map_err(TenantLockIndexError::from)?;
                // Implementation for retrieving available locks from PostgreSQL
                sqlx::query!(
                    "UPDATE tenants SET index_sequence_position = $1 WHERE id = $2",
                    model.index_sequence_position,
                    tenant_id.as_ref()
                )
                .execute(conn)
                .await
                .map_err(TenantLockIndexError::from)?;

                Ok(())
            }
            _ => Err(TenantLockIndexError::InvalidConnection.into()),
        }
    }
}
