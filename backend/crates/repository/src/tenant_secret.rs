//! Storage for the secrets of a tenant that the server has to read back.
//!
//! A secret is stored encrypted under one version of its tenant's key (see
//! [`crate::tenant_key`]). This module stores and returns what the encryption
//! produced; it holds no key and never sees a secret itself.
//!
//! A secret is found by its tenant, its kind and its owner, and a tenant has
//! at most one secret of a kind per owner.
//!
//! See the `20261126120000_tenant_secret` migration.

use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::TenantId;

/// What a secret is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TenantSecretKind {
    /// The shared secret of a TOTP credential.
    MfaTotp,
    /// The client secret the server authenticates with to an identity
    /// provider.
    IdpClientSecret,
}

impl TenantSecretKind {
    /// The kind as the `kind` column stores it.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            TenantSecretKind::MfaTotp => "mfa_totp",
            TenantSecretKind::IdpClientSecret => "idp_client_secret",
        }
    }
}

/// A secret as stored: encrypted under version `key_version` of its tenant's
/// key.
#[derive(sqlx::FromRow, Clone, Debug, PartialEq, Eq)]
pub struct TenantSecret {
    pub key_version: i32,
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
}

pub trait TenantSecretProvider {
    /// Stores `secret` as the `kind` secret of `owner`, in place of the one
    /// it has.
    ///
    /// Fails when `tenant` has no key of `secret.key_version`.
    fn put_tenant_secret(
        &self,
        tenant: &TenantId,
        kind: TenantSecretKind,
        owner: &str,
        secret: &TenantSecret,
    ) -> impl std::future::Future<Output = Result<(), OperationOutcomeError>> + Send;

    /// The `kind` secret of `owner`, or `None` when it has none.
    fn tenant_secret(
        &self,
        tenant: &TenantId,
        kind: TenantSecretKind,
        owner: &str,
    ) -> impl std::future::Future<Output = Result<Option<TenantSecret>, OperationOutcomeError>> + Send;

    /// Removes the `kind` secret of `owner`. `false` when it had none.
    fn delete_tenant_secret(
        &self,
        tenant: &TenantId,
        kind: TenantSecretKind,
        owner: &str,
    ) -> impl std::future::Future<Output = Result<bool, OperationOutcomeError>> + Send;
}
