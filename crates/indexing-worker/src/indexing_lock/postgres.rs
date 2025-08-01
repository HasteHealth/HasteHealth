use crate::indexing_lock::{IndexLockProvider, TenantLockIndex};
use oxidized_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use sqlx::{Postgres, QueryBuilder, Transaction};

pub struct PostgresIndexLockProvider<'a, 'b> {
    connection: &'a mut Transaction<'b, Postgres>,
}
impl<'a, 'b> PostgresIndexLockProvider<'a, 'b> {
    pub fn new(connection: &'a mut Transaction<'b, Postgres>) -> Self {
        PostgresIndexLockProvider { connection }
    }

    pub fn set_connection(&mut self, connection: &'a mut Transaction<'b, Postgres>) {
        self.connection = connection;
    }
}

#[derive(OperationOutcomeError, Debug)]
pub enum TenantLockIndexError {
    #[fatal(code = "exception", diagnostic = "SQL error occurred {arg0}")]
    SQLError(#[from] sqlx::Error),
}

impl<'a, 'b> IndexLockProvider for PostgresIndexLockProvider<'a, 'b> {
    async fn get_available(
        &mut self,
        tenants: Vec<String>,
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
            .fetch_all(&mut **self.connection)
            .await
            .map_err(TenantLockIndexError::from)?;

        Ok(res)
    }

    async fn update(
        &mut self,
        tenant_id: String,
        next_position: usize,
    ) -> Result<(), OperationOutcomeError> {
        // Implementation for updating a lock in PostgreSQL
        sqlx::query!(
            "UPDATE tenants SET index_sequence_position = $1 WHERE id = $2",
            next_position as i64,
            tenant_id
        )
        .execute(&mut **self.connection)
        .await
        .map_err(TenantLockIndexError::from)?;

        Ok(())
    }
}
