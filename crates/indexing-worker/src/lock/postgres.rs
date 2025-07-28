use crate::lock::{Lock, LockId, LockProvider, LockType};
use oxidized_fhir_operation_error::OperationOutcomeError;
use sqlx::{Execute, Postgres, QueryBuilder};

pub struct PostgresLockProvider {
    connection: sqlx::PgConnection,
}
impl PostgresLockProvider {
    pub fn new(connection: sqlx::PgConnection) -> Self {
        PostgresLockProvider { connection }
    }
}

impl LockProvider for PostgresLockProvider {
    fn get_available(
        &self,
        lock_type: LockType,
        lock_ids: Vec<LockId>,
    ) -> Result<Vec<Lock>, OperationOutcomeError> {
        // Implementation for retrieving available locks from PostgreSQL
        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new("SELECT * FROM locks WHERE lock_type = ");

        query_builder.push_bind(lock_type.as_ref());
        query_builder.push(" AND lock_id IN (");

        let mut separated = query_builder.separated(", ");
        for lock_id in lock_ids.iter() {
            separated.push_bind(lock_id.as_ref());
        }

        separated.push_unseparated(") ");

        let query = query_builder.build();
        println!("Executing query: '{:?}'", query.sql());
        Ok(vec![])
    }

    fn update(
        &self,
        lock_type: LockType,
        lock_id: LockId,
        value: Lock,
    ) -> Result<(), OperationOutcomeError> {
        // Implementation for updating a lock in PostgreSQL
        unimplemented!()
    }

    fn create(&self, lock: Lock) -> Result<Lock, OperationOutcomeError> {
        // Implementation for creating a new lock in PostgreSQL
        unimplemented!()
    }
}
