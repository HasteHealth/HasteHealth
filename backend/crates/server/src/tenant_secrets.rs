//! The secrets of a tenant that the server has to read back, such as the
//! shared secret of a TOTP credential.
//!
//! A secret is encrypted under its tenant's key (see [`crate::tenant_keys`])
//! and stored in the `tenant_secret` table (see
//! `haste_repository::tenant_secret`). It is found by its tenant, its kind
//! and its owner: a tenant has at most one secret of a kind per owner.
//!
//! The encryption binds a secret to those three, its label. The label is not
//! stored. Reading a secret builds the label again from what was asked for,
//! so a stored secret that ends up in another row does not decrypt there.

use crate::tenant_keys::{TenantEncryptionResult, TenantKeys};
use haste_encryption::{Secret, traits::EncryptionResult};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::TenantId;
use haste_repository::{
    tenant_key::TenantKeyProvider,
    tenant_secret::{TenantSecret, TenantSecretKind, TenantSecretProvider},
};

/// Says that a label is one of this store, and not of something else that
/// encrypts under a tenant's key.
const LABEL_DOMAIN: &str = "tenant_secret";

/// The label of a secret: its tenant, kind and owner, each preceded by its
/// length, so that no two secrets share a label.
///
/// A change to what this returns makes every stored secret unreadable.
fn label(tenant: &TenantId, kind: TenantSecretKind, owner: &str) -> Vec<u8> {
    let parts = [LABEL_DOMAIN, tenant.as_ref(), kind.as_str(), owner];

    let mut label = Vec::with_capacity(parts.iter().map(|part| 8 + part.len()).sum());
    for part in parts {
        label.extend_from_slice(&(part.len() as u64).to_be_bytes());
        label.extend_from_slice(part.as_bytes());
    }
    label
}

/// Stores, reads and removes the secrets of tenants.
///
/// Secrets are read and written through `secret_store`. When that is a
/// transaction, a secret is stored or removed with the rest of the
/// transaction. The tenant's key is not: `tenant_keys` creates a tenant's
/// first key outside of it.
pub struct TenantSecrets<'a, KeyStore, SecretStore> {
    tenant_keys: &'a TenantKeys<KeyStore>,
    secret_store: &'a SecretStore,
}

impl<'a, KeyStore, SecretStore> TenantSecrets<'a, KeyStore, SecretStore>
where
    KeyStore: TenantKeyProvider + Send + Sync + 'static,
    SecretStore: TenantSecretProvider + Sync,
{
    #[must_use]
    pub fn new(tenant_keys: &'a TenantKeys<KeyStore>, secret_store: &'a SecretStore) -> Self {
        Self {
            tenant_keys,
            secret_store,
        }
    }

    /// Stores `secret` as the `kind` secret of `owner`, in place of the one
    /// it has, encrypted under the active key of `tenant`.
    ///
    /// # Errors
    ///
    /// Fails when the secret cannot be encrypted under the tenant's key (see
    /// [`TenantKeys::encrypt`]) or cannot be stored.
    pub async fn put(
        &self,
        tenant: &TenantId,
        kind: TenantSecretKind,
        owner: &str,
        secret: &[u8],
    ) -> Result<(), OperationOutcomeError> {
        let TenantEncryptionResult {
            key_version,
            encrypted,
        } = self
            .tenant_keys
            .encrypt(tenant, secret, &label(tenant, kind, owner))
            .await?;

        self.secret_store
            .put_tenant_secret(
                tenant,
                kind,
                owner,
                &TenantSecret {
                    key_version,
                    nonce: encrypted.nonce,
                    ciphertext: encrypted.ciphertext,
                },
            )
            .await
    }

    /// The `kind` secret of `owner`, or `None` when it has none.
    ///
    /// # Errors
    ///
    /// Fails when the secret cannot be read, or when it does not decrypt: the
    /// tenant's key is not available (see [`TenantKeys::decrypt`]), or what
    /// is stored was altered or was stored for another tenant, kind or owner.
    pub async fn get(
        &self,
        tenant: &TenantId,
        kind: TenantSecretKind,
        owner: &str,
    ) -> Result<Option<Secret>, OperationOutcomeError> {
        let Some(stored) = self.secret_store.tenant_secret(tenant, kind, owner).await? else {
            return Ok(None);
        };

        let key_version = stored.key_version;
        let secret = self
            .tenant_keys
            .decrypt(
                tenant,
                key_version,
                &EncryptionResult {
                    nonce: stored.nonce,
                    ciphertext: stored.ciphertext,
                },
                &label(tenant, kind, owner),
            )
            .await
            .inspect_err(|_| {
                tracing::error!(
                    tenant = %tenant,
                    kind = kind.as_str(),
                    owner,
                    key_version,
                    "A stored tenant secret could not be decrypted"
                );
            })?;

        Ok(Some(Secret::new(secret)))
    }

    /// Removes the `kind` secret of `owner`. `false` when it had none.
    ///
    /// # Errors
    ///
    /// Fails when the secret cannot be removed.
    pub async fn delete(
        &self,
        tenant: &TenantId,
        kind: TenantSecretKind,
        owner: &str,
    ) -> Result<bool, OperationOutcomeError> {
        self.secret_store
            .delete_tenant_secret(tenant, kind, owner)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tenant_keys::tests::{MemoryStore, new_stored_key, tenant, tenant_keys};
    use TenantSecretKind::{IdpClientSecret, MfaTotp};
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    type SecretId = (String, TenantSecretKind, String);

    /// The `tenant_secret` table in memory.
    #[derive(Default)]
    struct MemorySecretStore {
        secrets: Mutex<HashMap<SecretId, TenantSecret>>,
    }

    impl MemorySecretStore {
        fn id(tenant: &TenantId, kind: TenantSecretKind, owner: &str) -> SecretId {
            (tenant.as_ref().to_string(), kind, owner.to_string())
        }

        fn stored(&self, tenant: &TenantId, kind: TenantSecretKind, owner: &str) -> TenantSecret {
            self.secrets.lock().unwrap()[&Self::id(tenant, kind, owner)].clone()
        }

        fn store(
            &self,
            tenant: &TenantId,
            kind: TenantSecretKind,
            owner: &str,
            secret: TenantSecret,
        ) {
            let mut secrets = self.secrets.lock().unwrap();
            secrets.insert(Self::id(tenant, kind, owner), secret);
        }

        fn len(&self) -> usize {
            self.secrets.lock().unwrap().len()
        }
    }

    impl TenantSecretProvider for MemorySecretStore {
        async fn put_tenant_secret(
            &self,
            tenant: &TenantId,
            kind: TenantSecretKind,
            owner: &str,
            secret: &TenantSecret,
        ) -> Result<(), OperationOutcomeError> {
            self.store(tenant, kind, owner, secret.clone());
            Ok(())
        }

        async fn tenant_secret(
            &self,
            tenant: &TenantId,
            kind: TenantSecretKind,
            owner: &str,
        ) -> Result<Option<TenantSecret>, OperationOutcomeError> {
            let secrets = self.secrets.lock().unwrap();
            Ok(secrets.get(&Self::id(tenant, kind, owner)).cloned())
        }

        async fn delete_tenant_secret(
            &self,
            tenant: &TenantId,
            kind: TenantSecretKind,
            owner: &str,
        ) -> Result<bool, OperationOutcomeError> {
            let mut secrets = self.secrets.lock().unwrap();
            Ok(secrets.remove(&Self::id(tenant, kind, owner)).is_some())
        }
    }

    async fn get(
        secrets: &TenantSecrets<'_, MemoryStore, MemorySecretStore>,
        tenant: &TenantId,
        kind: TenantSecretKind,
        owner: &str,
    ) -> Option<Vec<u8>> {
        secrets
            .get(tenant, kind, owner)
            .await
            .unwrap()
            .map(|secret| secret.expose_bytes().to_vec())
    }

    #[tokio::test]
    async fn a_secret_is_stored_encrypted_and_read_back() {
        let (keys, secret_store) = (
            tenant_keys(&Arc::new(MemoryStore::default())),
            MemorySecretStore::default(),
        );
        let secrets = TenantSecrets::new(&keys, &secret_store);
        let acme = tenant("acme");

        assert_eq!(get(&secrets, &acme, MfaTotp, "credential-1").await, None);
        secrets
            .put(&acme, MfaTotp, "credential-1", b"totp-secret")
            .await
            .unwrap();

        let stored = secret_store.stored(&acme, MfaTotp, "credential-1");
        assert_eq!(stored.key_version, 1);
        assert_ne!(stored.ciphertext, b"totp-secret");
        assert_eq!(
            get(&secrets, &acme, MfaTotp, "credential-1").await.unwrap(),
            b"totp-secret"
        );
    }

    #[tokio::test]
    async fn a_put_replaces_the_secret() {
        let (keys, secret_store) = (
            tenant_keys(&Arc::new(MemoryStore::default())),
            MemorySecretStore::default(),
        );
        let secrets = TenantSecrets::new(&keys, &secret_store);
        let acme = tenant("acme");

        for secret in [&b"first"[..], b"second"] {
            secrets
                .put(&acme, IdpClientSecret, "idp-1", secret)
                .await
                .unwrap();
        }

        assert_eq!(secret_store.len(), 1);
        assert_eq!(
            get(&secrets, &acme, IdpClientSecret, "idp-1")
                .await
                .unwrap(),
            b"second"
        );
    }

    #[tokio::test]
    async fn a_deleted_secret_is_gone() {
        let (keys, secret_store) = (
            tenant_keys(&Arc::new(MemoryStore::default())),
            MemorySecretStore::default(),
        );
        let secrets = TenantSecrets::new(&keys, &secret_store);
        let acme = tenant("acme");
        secrets
            .put(&acme, MfaTotp, "credential-1", b"totp-secret")
            .await
            .unwrap();

        assert!(
            secrets
                .delete(&acme, MfaTotp, "credential-1")
                .await
                .unwrap()
        );

        assert_eq!(get(&secrets, &acme, MfaTotp, "credential-1").await, None);
        assert!(
            !secrets
                .delete(&acme, MfaTotp, "credential-1")
                .await
                .unwrap()
        );
    }

    /// The same owner has a secret per kind, and the same kind and owner a
    /// secret per tenant.
    #[tokio::test]
    async fn secrets_are_kept_apart_by_tenant_kind_and_owner() {
        let (keys, secret_store) = (
            tenant_keys(&Arc::new(MemoryStore::default())),
            MemorySecretStore::default(),
        );
        let secrets = TenantSecrets::new(&keys, &secret_store);
        let stored = [
            ("acme", MfaTotp, "one", &b"acme-totp-one"[..]),
            ("acme", MfaTotp, "two", b"acme-totp-two"),
            ("acme", IdpClientSecret, "one", b"acme-idp-one"),
            ("globex", MfaTotp, "one", b"globex-totp-one"),
        ];
        for (tenant_id, kind, owner, secret) in stored {
            secrets
                .put(&tenant(tenant_id), kind, owner, secret)
                .await
                .unwrap();
        }

        for (tenant_id, kind, owner, secret) in stored {
            assert_eq!(
                get(&secrets, &tenant(tenant_id), kind, owner)
                    .await
                    .unwrap(),
                secret
            );
        }
    }

    /// What is stored for one owner, kind or tenant is of no use in the row
    /// of another.
    #[tokio::test]
    async fn a_stored_secret_moved_to_another_row_does_not_decrypt() {
        let (keys, secret_store) = (
            tenant_keys(&Arc::new(MemoryStore::default())),
            MemorySecretStore::default(),
        );
        let secrets = TenantSecrets::new(&keys, &secret_store);
        let (acme, globex) = (tenant("acme"), tenant("globex"));
        secrets
            .put(&acme, MfaTotp, "credential-1", b"totp-secret")
            .await
            .unwrap();
        // Globex has a key of the same version.
        secrets
            .put(&globex, MfaTotp, "credential-9", b"other")
            .await
            .unwrap();
        let stored = secret_store.stored(&acme, MfaTotp, "credential-1");

        for (to_tenant, to_kind, to_owner) in [
            (&acme, MfaTotp, "credential-2"),
            (&acme, IdpClientSecret, "credential-1"),
            (&globex, MfaTotp, "credential-1"),
        ] {
            secret_store.store(to_tenant, to_kind, to_owner, stored.clone());

            assert!(
                secrets.get(to_tenant, to_kind, to_owner).await.is_err(),
                "{to_tenant} {to_kind:?} {to_owner}"
            );
        }
        // Where it was stored, it still decrypts.
        assert_eq!(
            get(&secrets, &acme, MfaTotp, "credential-1").await.unwrap(),
            b"totp-secret"
        );
    }

    #[tokio::test]
    async fn an_altered_secret_does_not_decrypt() {
        let (keys, secret_store) = (
            tenant_keys(&Arc::new(MemoryStore::default())),
            MemorySecretStore::default(),
        );
        let secrets = TenantSecrets::new(&keys, &secret_store);
        let acme = tenant("acme");
        secrets
            .put(&acme, MfaTotp, "credential-1", b"totp-secret")
            .await
            .unwrap();

        let mut altered = secret_store.stored(&acme, MfaTotp, "credential-1");
        altered.ciphertext[0] ^= 1;
        secret_store.store(&acme, MfaTotp, "credential-1", altered);

        assert!(secrets.get(&acme, MfaTotp, "credential-1").await.is_err());
    }

    /// After a rotation a secret stays readable under the key version it was
    /// stored with, and a new secret goes under the new one.
    #[tokio::test]
    async fn a_secret_is_read_with_the_key_version_it_was_stored_under() {
        let key_store = Arc::new(MemoryStore::default());
        let secret_store = MemorySecretStore::default();
        let acme = tenant("acme");
        let keys = tenant_keys(&key_store);
        TenantSecrets::new(&keys, &secret_store)
            .put(&acme, MfaTotp, "before", b"old-secret")
            .await
            .unwrap();
        key_store.rotate_to(&acme, new_stored_key(&acme, 2).await);

        // A server that starts after the rotation.
        let keys = tenant_keys(&key_store);
        let secrets = TenantSecrets::new(&keys, &secret_store);
        secrets
            .put(&acme, MfaTotp, "after", b"new-secret")
            .await
            .unwrap();

        assert_eq!(secret_store.stored(&acme, MfaTotp, "before").key_version, 1);
        assert_eq!(secret_store.stored(&acme, MfaTotp, "after").key_version, 2);
        assert_eq!(
            get(&secrets, &acme, MfaTotp, "before").await.unwrap(),
            b"old-secret"
        );
        assert_eq!(
            get(&secrets, &acme, MfaTotp, "after").await.unwrap(),
            b"new-secret"
        );
    }

    /// Nothing is stored when the secret cannot be encrypted.
    #[tokio::test]
    async fn without_a_platform_key_nothing_is_stored() {
        let keys = TenantKeys::new(Arc::new(MemoryStore::default()), None);
        let secret_store = MemorySecretStore::default();
        let secrets = TenantSecrets::new(&keys, &secret_store);

        assert!(
            secrets
                .put(&tenant("acme"), MfaTotp, "credential-1", b"totp-secret")
                .await
                .is_err()
        );
        assert_eq!(secret_store.len(), 0);
    }

    /// Stored secrets depend on the label staying as it is.
    #[test]
    fn the_label_is_stable() {
        let mut expected = Vec::new();
        for part in ["tenant_secret", "acme", "mfa_totp", "credential-1"] {
            expected.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, u8::try_from(part.len()).unwrap()]);
            expected.extend_from_slice(part.as_bytes());
        }

        assert_eq!(label(&tenant("acme"), MfaTotp, "credential-1"), expected);
    }

    /// Parts that join to the same text are still different labels.
    #[test]
    fn labels_do_not_collide() {
        let acme = tenant("acme");
        let labels = [
            label(&acme, MfaTotp, "ab"),
            label(&acme, MfaTotp, "a"),
            label(&tenant("acmea"), MfaTotp, "b"),
            label(&acme, IdpClientSecret, "ab"),
            label(&tenant("globex"), MfaTotp, "ab"),
        ];

        for (index, label) in labels.iter().enumerate() {
            assert!(!labels[index + 1..].contains(label));
        }
    }
}
