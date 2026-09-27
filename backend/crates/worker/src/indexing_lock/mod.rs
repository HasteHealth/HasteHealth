use haste_fhir_operation_error::OperationOutcomeError;
use haste_repository::types::SearchIndexBackend;

pub mod postgres;

pub trait IndexLockProvider<ID, Model> {
    /// Retrieves available locks skipping over locked rows.
    /// Sets available locks to be locked until transaction is committed.
    /// * `backend` - Which search backend's locks to select (locks are
    ///   independent per backend, so an Elasticsearch worker and a Postgres
    ///   search worker never contend over the same row)
    /// * `project_ids` - Ids of the (tenant, project) locks to select
    fn get_available_locks(
        &self,
        backend: SearchIndexBackend,
        project_ids: Vec<&ID>,
    ) -> impl std::future::Future<Output = Result<Vec<Model>, OperationOutcomeError>> + Send;
    fn update_lock(
        &self,
        backend: SearchIndexBackend,
        project_id: &ID,
        model: Model,
    ) -> impl std::future::Future<Output = Result<(), OperationOutcomeError>> + Send;
}
