use crate::lock::{Lock, LockId, LockKind, LockProvider};
use oxidized_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use sqlx::{Postgres, QueryBuilder, Transaction};

pub struct PostgresLockProvider<'a, 'b> {
    connection: &'a mut Transaction<'b, Postgres>,
}
impl<'a, 'b> PostgresLockProvider<'a, 'b> {
    pub fn new(connection: &'a mut Transaction<'b, Postgres>) -> Self {
        PostgresLockProvider { connection }
    }

    pub fn set_connection(&mut self, connection: &'a mut Transaction<'b, Postgres>) {
        self.connection = connection;
    }
}

#[derive(OperationOutcomeError, Debug)]
pub enum LockError {
    #[fatal(code = "exception", diagnostic = "SQL error occurred {arg0}")]
    SQLError(#[from] sqlx::Error),
}

impl<'a, 'b> LockProvider for PostgresLockProvider<'a, 'b> {
    async fn get_available(
        &mut self,
        kind: LockKind,
        lock_ids: Vec<LockId>,
    ) -> Result<Vec<Lock>, OperationOutcomeError> {
        // Implementation for retrieving available locks from PostgreSQL
        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new("SELECT * FROM locks WHERE kind = ");

        query_builder.push_bind(kind);
        query_builder.push(" AND id IN (");

        let mut separated = query_builder.separated(", ");
        for lock_id in lock_ids.iter() {
            separated.push_bind(lock_id.as_ref());
        }

        separated.push_unseparated(") ");

        let query = query_builder.build_query_as();
        // println!("Executing query: '{:?}'", query.sql());
        let res = query
            .fetch_all(&mut **self.connection)
            .await
            .map_err(LockError::from)?;

        Ok(res)
    }

    async fn update(
        &mut self,
        _kind: LockKind,
        _lock_id: LockId,
        _value: Lock,
    ) -> Result<(), OperationOutcomeError> {
        // Implementation for updating a lock in PostgreSQL
        unimplemented!()
    }

    async fn create(&mut self, _lock: Lock) -> Result<Lock, OperationOutcomeError> {
        // Implementation for creating a new lock in PostgreSQL
        unimplemented!()
    }
}
