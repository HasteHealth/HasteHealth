use crate::indexing_lock::{IndexLockProvider, TenantLockIndex};
use oxidized_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use sqlx::{Postgres, QueryBuilder, Transaction};

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
}

impl<'b> IndexLockProvider<Transaction<'b, Postgres>> for PostgresIndexLockProvider {
    async fn get_available(
        &self,
        conn: &mut Transaction<'b, Postgres>,
        tenants: Vec<&str>,
    ) -> Result<Vec<TenantLockIndex>, OperationOutcomeError> {
        // Implementation for retrieving available locks from PostgreSQL

        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new("SELECT id, index_sequence_position FROM tenants WHERE id IN ( ");

        let mut separated = query_builder.separated(", ");
        for tenant_id in tenants.iter() {
            separated.push_bind(tenant_id);
        }

        separated.push_unseparated(") FOR UPDATE SKIP LOCKED");

        let query = query_builder.build_query_as();
        // println!("Executing query: '{:?}'", query.sql());
        let res = query
            .fetch_all(&mut **conn)
            .await
            .map_err(TenantLockIndexError::from)?;

        Ok(res)
    }

    async fn update(
        &self,
        conn: &mut Transaction<'b, Postgres>,
        tenant_id: &str,
        next_position: usize,
    ) -> Result<(), OperationOutcomeError> {
        // Implementation for updating a lock in PostgreSQL
        sqlx::query!(
            "UPDATE tenants SET index_sequence_position = $1 WHERE id = $2",
            next_position as i64,
            tenant_id
        )
        .execute(&mut **conn)
        .await
        .map_err(TenantLockIndexError::from)?;

        Ok(())
    }
}
