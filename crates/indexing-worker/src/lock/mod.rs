use oxidized_fhir_operation_error::OperationOutcomeError;

pub mod postgres;

pub enum LockType {
    IndexingPosition,
}

pub struct LockId(String);
impl LockId {
    pub fn new(id: String) -> Self {
        LockId(id)
    }
}
impl AsRef<str> for LockId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

pub struct Lock {
    lock_id: LockId,
    lock_type: LockType,
    value: String,
}

pub trait LockProvider {
    /// Retrieves available locks skipping over locked rows.
    /// Sets available locks to be locked until transaction is committed.
    /// * `lock_type` - Lock type to select
    /// * `lock_ids` - Ids of locks to select
    fn get_available(
        &self,
        lock_type: LockType,
        lock_ids: Vec<LockId>,
    ) -> Result<Vec<Lock>, OperationOutcomeError>;
    fn update(
        &self,
        lock_type: LockType,
        lock_id: LockId,
        value: Lock,
    ) -> Result<(), OperationOutcomeError>;
    fn create(&self, lock: Lock) -> Result<Lock, OperationOutcomeError>;
}
