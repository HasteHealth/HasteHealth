use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_jwt::TenantId;

pub mod postgres;

#[derive(sqlx::FromRow, Debug)]
pub struct TenantLockIndex {
    pub id: String,
    pub index_sequence_position: i64,
}

pub trait IndexLockProvider {
    /// Retrieves available locks skipping over locked rows.
    /// Sets available locks to be locked until transaction is committed.
    /// * `kind` - Lock kind to select
    /// * `lock_ids` - Ids of locks to select
    fn get_available_locks(
        &self,
        tenant_ids: Vec<&TenantId>,
    ) -> impl std::future::Future<Output = Result<Vec<TenantLockIndex>, OperationOutcomeError>> + Send;
    fn update_lock(
        &self,
        tenant_id: &str,
        next_position: usize,
    ) -> impl std::future::Future<Output = Result<(), OperationOutcomeError>> + Send;
}
