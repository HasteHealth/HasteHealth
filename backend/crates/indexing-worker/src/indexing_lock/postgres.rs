use crate::indexing_lock::{IndexLockProvider, TenantLockIndex};
use oxidized_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use oxidized_repository::pg::PGConnection;
use sqlx::{Acquire, Postgres, QueryBuilder, Transaction};

pub struct PostgresIndexLockProvider();

impl PostgresIndexLockProvider {
    pub fn new() -> Self {
        Self()
    }
}

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

impl IndexLockProvider for PGConnection {
    async fn get_available_locks(
        &self,
        tenants: Vec<&str>,
    ) -> Result<Vec<TenantLockIndex>, OperationOutcomeError> {
        match self {
            PGConnection::Transaction(tx) => {
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
                    separated.push_bind(tenant_id);
                }

                separated.push_unseparated(") FOR UPDATE SKIP LOCKED");

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
        tenant_id: &str,
        next_position: usize,
    ) -> Result<(), OperationOutcomeError> {
        match self {
            PGConnection::Transaction(tx) => {
                let mut tx = tx.lock().await;
                let conn = (&mut (*tx))
                    .acquire()
                    .await
                    .map_err(TenantLockIndexError::from)?;
                // Implementation for retrieving available locks from PostgreSQL
                sqlx::query!(
                    "UPDATE tenants SET index_sequence_position = $1 WHERE id = $2",
                    next_position as i64,
                    tenant_id
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
