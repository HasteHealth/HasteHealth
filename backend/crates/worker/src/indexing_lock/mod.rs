use haste_fhir_operation_error::OperationOutcomeError;

pub mod postgres;

pub trait IndexLockProvider<ID, Model> {
    /// Retrieves available locks skipping over locked rows.
    /// Sets available locks to be locked until transaction is committed.
    /// * `kind` - Lock kind to select
    /// * `lock_ids` - Ids of locks to select
    fn get_available_locks(
        &self,
        tenant_ids: Vec<&ID>,
    ) -> impl std::future::Future<Output = Result<Vec<Model>, OperationOutcomeError>> + Send;
    fn update_lock(
        &self,
        tenant_id: &ID,
        model: Model,
    ) -> impl std::future::Future<Output = Result<(), OperationOutcomeError>> + Send;
}
