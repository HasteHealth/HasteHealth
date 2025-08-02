#![allow(unused)]
use oxidized_fhir_operation_error::OperationOutcomeError;

pub mod postgres;

#[derive(sqlx::FromRow, Debug)]
pub struct TenantLockIndex {
    pub id: String,
    pub index_sequence_position: i64,
}

pub trait IndexLockProvider<Connection> {
    /// Retrieves available locks skipping over locked rows.
    /// Sets available locks to be locked until transaction is committed.
    /// * `kind` - Lock kind to select
    /// * `lock_ids` - Ids of locks to select
    fn get_available(
        &self,
        conn: &mut Connection,
        tenant_ids: Vec<&str>,
    ) -> impl std::future::Future<Output = Result<Vec<TenantLockIndex>, OperationOutcomeError>> + Send;
    fn update(
        &self,
        conn: &mut Connection,
        tenant_id: &str,
        next_position: usize,
    ) -> impl std::future::Future<Output = Result<(), OperationOutcomeError>> + Send;
}
