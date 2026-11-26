//! Storage for the keys that encrypt a tenant's secrets.
//!
//! A tenant has its own key, in versions: the active one encrypts, and an
//! older one still decrypts what it encrypted. A key is stored wrapped, that
//! is encrypted by a platform key the database never holds, so a row here is
//! of no use without that platform key.
//!
//! See the `20261125120000_tenant_key` migration.

use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::TenantId;

/// One version of a tenant's key, as stored.
#[derive(sqlx::FromRow, Clone, Debug, PartialEq, Eq)]
pub struct TenantKey {
    pub version: i32,
    /// The key, encrypted by the platform key `wrapped_by` names.
    pub wrapped_key: Vec<u8>,
    pub wrapped_by: String,
}

pub trait TenantKeyProvider {
    /// The key that encrypts for `tenant` now, or `None` when it has none.
    fn active_tenant_key(
        &self,
        tenant: &TenantId,
    ) -> impl std::future::Future<Output = Result<Option<TenantKey>, OperationOutcomeError>> + Send;

    /// The key of `tenant` at `version`, active or not.
    fn tenant_key(
        &self,
        tenant: &TenantId,
        version: i32,
    ) -> impl std::future::Future<Output = Result<Option<TenantKey>, OperationOutcomeError>> + Send;

    /// Stores `wrapped_key` as the first key of `tenant`, unless it has one
    /// already, and answers the key that is active afterwards. When two
    /// callers race, both get the key of the one that stored first: the other
    /// key was never used and is dropped.
    ///
    /// `None` when the tenant has keys and none is active.
    ///
    /// Refused inside a transaction: the caller keeps the key it gets, and a
    /// rollback would remove the key while what it encrypted stayed.
    fn create_first_tenant_key(
        &self,
        tenant: &TenantId,
        wrapped_key: &[u8],
        wrapped_by: &str,
    ) -> impl std::future::Future<Output = Result<Option<TenantKey>, OperationOutcomeError>> + Send;
}
