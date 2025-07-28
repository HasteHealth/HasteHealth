use oxidized_fhir_operation_error::OperationOutcomeError;

pub mod postgres;

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, serde::Deserialize, serde::Serialize)]
#[sqlx(type_name = "lock_type", rename_all = "lowercase")]
pub enum LockType {
    IndexingPosition,
}
impl AsRef<str> for LockType {
    fn as_ref(&self) -> &str {
        match self {
            LockType::IndexingPosition => "indexing_position",
        }
    }
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
impl From<&str> for LockId {
    fn from(id: &str) -> Self {
        LockId::new(id.to_string())
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
    async fn get_available(
        &mut self,
        lock_type: LockType,
        lock_ids: Vec<LockId>,
    ) -> Result<Vec<Lock>, OperationOutcomeError>;
    async fn update(
        &mut self,
        lock_type: LockType,
        lock_id: LockId,
        value: Lock,
    ) -> Result<(), OperationOutcomeError>;
    async fn create(&mut self, lock: Lock) -> Result<Lock, OperationOutcomeError>;
}
