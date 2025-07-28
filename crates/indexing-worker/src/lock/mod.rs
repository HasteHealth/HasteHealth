use oxidized_fhir_operation_error::OperationOutcomeError;

pub mod postgres;

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, serde::Deserialize, serde::Serialize)]
#[sqlx(type_name = "lock_kind", rename_all = "lowercase")]
pub enum LockKind {
    System,
}
impl AsRef<str> for LockKind {
    fn as_ref(&self) -> &str {
        match self {
            LockKind::System => "system",
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

#[derive(sqlx::FromRow, Debug)]
pub struct Lock {
    pub tenant: String,
    pub kind: LockKind,
    pub id: String,
    pub position: i64,
}

pub trait LockProvider {
    /// Retrieves available locks skipping over locked rows.
    /// Sets available locks to be locked until transaction is committed.
    /// * `kind` - Lock kind to select
    /// * `lock_ids` - Ids of locks to select
    fn get_available(
        &mut self,
        kind: LockKind,
        lock_ids: Vec<LockId>,
    ) -> impl std::future::Future<Output = Result<Vec<Lock>, OperationOutcomeError>> + Send;
    fn update(
        &mut self,
        kind: LockKind,
        lock_id: LockId,
        value: Lock,
    ) -> impl std::future::Future<Output = Result<(), OperationOutcomeError>> + Send;
    fn create(
        &mut self,
        lock: Lock,
    ) -> impl std::future::Future<Output = Result<Lock, OperationOutcomeError>> + Send;
}
