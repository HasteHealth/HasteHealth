//! Encryption under a tenant's own key.
//!
//! Each tenant has a key of its own for the secrets the server has to read
//! back. The key is stored wrapped by a platform key (see
//! `haste_repository::tenant_key`), so reading the database does not give it
//! away. A tenant gets its key the first time something is encrypted for it.
//!
//! Keys have versions. A value is encrypted under the tenant's active key and
//! is stored with that key's version, which is what decrypts it later, also
//! once another version has become the active one.

use haste_encryption::{
    Encryptor as _, KeyWrapper, encryption::aes::AesGcmEncryptor, traits::EncryptionResult,
};
use haste_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use haste_jwt::TenantId;
use haste_repository::tenant_key::{TenantKey, TenantKeyProvider};
use moka::future::Cache;
use std::{sync::Arc, time::Duration};

/// How long an unwrapped key stays in memory. Until it is dropped, nothing
/// asks the platform key (a network call, with a KMS) or the database.
///
/// It is also how long a server goes on encrypting under a key after another
/// server has made a newer version the active one.
const UNWRAPPED_KEY_LIFETIME: Duration = Duration::from_mins(5);

/// The most unwrapped keys held at once, per cache.
const UNWRAPPED_KEY_CAPACITY: u64 = 10_000;

#[derive(OperationOutcomeError, Debug)]
pub enum TenantKeyError {
    #[error(
        code = "exception",
        diagnostic = "No platform key is configured to protect tenant keys."
    )]
    NotConfigured,
    /// The tenant has keys and none of them is active.
    #[fatal(code = "exception", diagnostic = "The tenant has no active key.")]
    NoActiveKey,
    #[fatal(
        code = "exception",
        diagnostic = "The tenant has no key of version {arg0}."
    )]
    UnknownVersion(i32),
    #[fatal(
        code = "exception",
        diagnostic = "The tenant's key is protected by a platform key other than the configured one."
    )]
    OtherPlatformKey,
}

/// A value encrypted for a tenant, with the version of the tenant's key that
/// decrypts it. Store the version next to the value.
pub struct TenantEncryptionResult {
    pub key_version: i32,
    pub encrypted: EncryptionResult,
}

/// A tenant's key, ready to use.
struct UnwrappedKey {
    version: i32,
    encryptor: AesGcmEncryptor,
}

/// Encrypts and decrypts with the keys of tenants.
pub struct TenantKeys<Store> {
    store: Arc<Store>,
    /// `None` when no platform key is configured: every use then fails.
    wrapper: Option<Arc<dyn KeyWrapper>>,
    /// The key that encrypts for a tenant.
    active: Cache<TenantId, Arc<UnwrappedKey>>,
    /// The keys that decrypt, by version.
    by_version: Cache<(TenantId, i32), Arc<UnwrappedKey>>,
}

/// A cache load hands its error to every waiting caller behind an `Arc`.
fn unshare(error: Arc<OperationOutcomeError>) -> OperationOutcomeError {
    Arc::try_unwrap(error)
        .unwrap_or_else(|shared| OperationOutcomeError::new(None, shared.outcome().clone()))
}

impl<Store: TenantKeyProvider + Send + Sync + 'static> TenantKeys<Store> {
    /// `store` is not a transaction: a key it creates has to outlast the
    /// request that needed it first.
    #[must_use]
    pub fn new(store: Arc<Store>, wrapper: Option<Arc<dyn KeyWrapper>>) -> Self {
        Self {
            store,
            wrapper,
            active: Cache::builder()
                .max_capacity(UNWRAPPED_KEY_CAPACITY)
                .time_to_live(UNWRAPPED_KEY_LIFETIME)
                .build(),
            by_version: Cache::builder()
                .max_capacity(UNWRAPPED_KEY_CAPACITY)
                .time_to_live(UNWRAPPED_KEY_LIFETIME)
                .build(),
        }
    }

    /// Encrypts `plaintext` under the active key of `tenant`, creating the
    /// tenant's first key when it has none, and binds the result to `aad`
    /// (see [`haste_encryption::Encryptor::encrypt_with_aad`]).
    ///
    /// # Errors
    ///
    /// Fails when no platform key is configured, when the tenant's key cannot
    /// be read, created or unwrapped, or when the encryption fails.
    pub async fn encrypt(
        &self,
        tenant: &TenantId,
        plaintext: &[u8],
        aad: &[u8],
    ) -> Result<TenantEncryptionResult, OperationOutcomeError> {
        let key = self
            .active
            .try_get_with_by_ref(tenant, self.load_active(tenant))
            .await
            .map_err(unshare)?;

        Ok(TenantEncryptionResult {
            key_version: key.version,
            encrypted: key.encryptor.encrypt_with_aad(plaintext, aad)?,
        })
    }

    /// Decrypts what [`Self::encrypt`] returned for `tenant` and `aad`, with
    /// the key version it was stored with.
    ///
    /// # Errors
    ///
    /// Fails when no platform key is configured, when the tenant has no key
    /// of `key_version` or it cannot be unwrapped, or when `encrypted` was
    /// altered or was encrypted for another tenant, key version or `aad`.
    pub async fn decrypt(
        &self,
        tenant: &TenantId,
        key_version: i32,
        encrypted: &EncryptionResult,
        aad: &[u8],
    ) -> Result<Vec<u8>, OperationOutcomeError> {
        let key = self
            .by_version
            .try_get_with(
                (tenant.clone(), key_version),
                self.load_version(tenant, key_version),
            )
            .await
            .map_err(unshare)?;

        key.encryptor.decrypt_with_aad(encrypted, aad)
    }

    fn wrapper(&self) -> Result<&dyn KeyWrapper, TenantKeyError> {
        self.wrapper.as_deref().ok_or(TenantKeyError::NotConfigured)
    }

    async fn load_active(
        &self,
        tenant: &TenantId,
    ) -> Result<Arc<UnwrappedKey>, OperationOutcomeError> {
        let wrapper = self.wrapper()?;

        let stored = if let Some(stored) = self.store.active_tenant_key(tenant).await? {
            stored
        } else {
            let key = AesGcmEncryptor::generate_key();
            let wrapped_key = wrapper
                .wrap_key(key.expose_bytes(), tenant.as_ref())
                .await?;

            // Another server may have stored a first key meanwhile; that one
            // is then the tenant's key, and the one generated here is dropped.
            let stored = self
                .store
                .create_first_tenant_key(tenant, &wrapped_key, wrapper.key_id())
                .await?
                .ok_or(TenantKeyError::NoActiveKey)?;

            if stored.wrapped_key == wrapped_key {
                tracing::info!(
                    tenant = %tenant,
                    version = stored.version,
                    wrapped_by = %stored.wrapped_by,
                    "Created the tenant's first key"
                );
            }
            stored
        };

        let key = self.unwrap(tenant, stored).await?;
        // What is encrypted under it is decrypted without unwrapping again.
        self.by_version
            .insert((tenant.clone(), key.version), key.clone())
            .await;

        Ok(key)
    }

    async fn load_version(
        &self,
        tenant: &TenantId,
        version: i32,
    ) -> Result<Arc<UnwrappedKey>, OperationOutcomeError> {
        self.wrapper()?;

        let stored = self
            .store
            .tenant_key(tenant, version)
            .await?
            .ok_or(TenantKeyError::UnknownVersion(version))?;

        self.unwrap(tenant, stored).await
    }

    async fn unwrap(
        &self,
        tenant: &TenantId,
        stored: TenantKey,
    ) -> Result<Arc<UnwrappedKey>, OperationOutcomeError> {
        let wrapper = self.wrapper()?;

        if stored.wrapped_by != wrapper.key_id() {
            tracing::error!(
                tenant = %tenant,
                version = stored.version,
                wrapped_by = %stored.wrapped_by,
                configured = %wrapper.key_id(),
                "The tenant's key is wrapped by a platform key other than the configured one"
            );
            return Err(TenantKeyError::OtherPlatformKey.into());
        }

        let key = wrapper
            .unwrap_key(&stored.wrapped_key, tenant.as_ref())
            .await?;

        Ok(Arc::new(UnwrappedKey {
            version: stored.version,
            encryptor: AesGcmEncryptor::new(key.expose_bytes())?,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use haste_encryption::{Secret, SecretsProvider, key_wrapper::local::LocalKeyWrapper};
    use std::{
        collections::HashMap,
        future::Future,
        pin::Pin,
        sync::{
            Mutex,
            atomic::{AtomicUsize, Ordering},
        },
    };

    /// Tenant keys in memory, with `retired` and `active` as the table has
    /// them, and a count of the reads and writes.
    #[derive(Default)]
    struct MemoryStore {
        keys: Mutex<HashMap<String, Vec<(TenantKey, bool)>>>,
        calls: AtomicUsize,
    }

    impl MemoryStore {
        fn calls(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }

        fn versions(&self, tenant: &TenantId) -> Vec<TenantKey> {
            let keys = self.keys.lock().unwrap();
            keys.get(tenant.as_ref())
                .map(|keys| keys.iter().map(|(key, _)| key.clone()).collect())
                .unwrap_or_default()
        }

        /// Retires the active key of `tenant` and makes `key` the active one.
        fn rotate_to(&self, tenant: &TenantId, key: TenantKey) {
            let mut keys = self.keys.lock().unwrap();
            let keys = keys.entry(tenant.as_ref().to_string()).or_default();
            for (_, active) in keys.iter_mut() {
                *active = false;
            }
            keys.push((key, true));
        }
    }

    impl TenantKeyProvider for MemoryStore {
        async fn active_tenant_key(
            &self,
            tenant: &TenantId,
        ) -> Result<Option<TenantKey>, OperationOutcomeError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let keys = self.keys.lock().unwrap();
            Ok(keys.get(tenant.as_ref()).and_then(|keys| {
                keys.iter()
                    .find(|(_, active)| *active)
                    .map(|(key, _)| key.clone())
            }))
        }

        async fn tenant_key(
            &self,
            tenant: &TenantId,
            version: i32,
        ) -> Result<Option<TenantKey>, OperationOutcomeError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let keys = self.keys.lock().unwrap();
            Ok(keys.get(tenant.as_ref()).and_then(|keys| {
                keys.iter()
                    .find(|(key, _)| key.version == version)
                    .map(|(key, _)| key.clone())
            }))
        }

        async fn create_first_tenant_key(
            &self,
            tenant: &TenantId,
            wrapped_key: &[u8],
            wrapped_by: &str,
        ) -> Result<Option<TenantKey>, OperationOutcomeError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let mut keys = self.keys.lock().unwrap();
            let keys = keys.entry(tenant.as_ref().to_string()).or_default();
            if keys.is_empty() {
                keys.push((
                    TenantKey {
                        version: 1,
                        wrapped_key: wrapped_key.to_vec(),
                        wrapped_by: wrapped_by.to_string(),
                    },
                    true,
                ));
            }
            Ok(keys
                .iter()
                .find(|(_, active)| *active)
                .map(|(key, _)| key.clone()))
        }
    }

    /// Platform keys by name, in memory.
    struct StaticSecrets;

    impl SecretsProvider for StaticSecrets {
        fn get_secret<'a>(
            &'a self,
            name: &'a str,
        ) -> Pin<Box<dyn Future<Output = Result<Secret, OperationOutcomeError>> + Send + 'a>>
        {
            Box::pin(async move {
                match name {
                    "PLATFORM_KEY" => Ok(Secret::new(vec![1u8; 32])),
                    "OTHER_PLATFORM_KEY" => Ok(Secret::new(vec![2u8; 32])),
                    other => Err(haste_encryption::EncryptionError::SecretNotFound(
                        other.to_string(),
                    )
                    .into()),
                }
            })
        }
    }

    fn wrapper(secret_name: &str) -> Arc<dyn KeyWrapper> {
        Arc::new(LocalKeyWrapper::new(
            Arc::new(StaticSecrets),
            secret_name.to_string(),
        ))
    }

    fn tenant_keys(store: &Arc<MemoryStore>) -> TenantKeys<MemoryStore> {
        TenantKeys::new(store.clone(), Some(wrapper("PLATFORM_KEY")))
    }

    fn tenant(id: &str) -> TenantId {
        TenantId::new(id.to_string()).unwrap()
    }

    /// A new key for `tenant` at `version`, wrapped as the server would.
    async fn new_stored_key(tenant: &TenantId, version: i32) -> TenantKey {
        let wrapper = wrapper("PLATFORM_KEY");
        TenantKey {
            version,
            wrapped_key: wrapper
                .wrap_key(
                    AesGcmEncryptor::generate_key().expose_bytes(),
                    tenant.as_ref(),
                )
                .await
                .unwrap(),
            wrapped_by: wrapper.key_id().to_string(),
        }
    }

    #[tokio::test]
    async fn the_first_encryption_creates_the_tenants_key() {
        let store = Arc::new(MemoryStore::default());
        let keys = tenant_keys(&store);
        let acme = tenant("acme");

        let result = keys.encrypt(&acme, b"totp-secret", b"label").await.unwrap();

        assert_eq!(result.key_version, 1);
        assert_ne!(result.encrypted.ciphertext, b"totp-secret");
        let stored = store.versions(&acme);
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].wrapped_by, "PLATFORM_KEY");

        let plaintext = keys
            .decrypt(&acme, result.key_version, &result.encrypted, b"label")
            .await
            .unwrap();
        assert_eq!(plaintext, b"totp-secret");
    }

    /// Another server, which shares only the store and the platform key,
    /// reads what this one wrote.
    #[tokio::test]
    async fn another_server_decrypts_with_the_stored_key() {
        let store = Arc::new(MemoryStore::default());
        let acme = tenant("acme");
        let result = tenant_keys(&store)
            .encrypt(&acme, b"totp-secret", b"label")
            .await
            .unwrap();

        let other_server = tenant_keys(&store);
        let plaintext = other_server
            .decrypt(&acme, result.key_version, &result.encrypted, b"label")
            .await
            .unwrap();
        assert_eq!(plaintext, b"totp-secret");

        // And it encrypts under the same key rather than a second one.
        let again = other_server
            .encrypt(&acme, b"totp-secret", b"label")
            .await
            .unwrap();
        assert_eq!(again.key_version, 1);
        assert_eq!(store.versions(&acme).len(), 1);
    }

    #[tokio::test]
    async fn another_label_does_not_decrypt() {
        let store = Arc::new(MemoryStore::default());
        let keys = tenant_keys(&store);
        let acme = tenant("acme");
        let result = keys
            .encrypt(&acme, b"totp-secret", b"user-1")
            .await
            .unwrap();

        assert!(
            keys.decrypt(&acme, result.key_version, &result.encrypted, b"user-2")
                .await
                .is_err()
        );
    }

    /// The same label and key version in another tenant is another key.
    #[tokio::test]
    async fn another_tenant_does_not_decrypt() {
        let store = Arc::new(MemoryStore::default());
        let keys = tenant_keys(&store);
        let (acme, globex) = (tenant("acme"), tenant("globex"));
        let for_acme = keys.encrypt(&acme, b"totp-secret", b"label").await.unwrap();
        let for_globex = keys
            .encrypt(&globex, b"totp-secret", b"label")
            .await
            .unwrap();
        assert_eq!(for_acme.key_version, for_globex.key_version);

        assert!(
            keys.decrypt(&globex, for_acme.key_version, &for_acme.encrypted, b"label")
                .await
                .is_err()
        );
    }

    /// A tenant's stored key moved to another tenant's row does not unwrap.
    #[tokio::test]
    async fn a_key_stored_for_another_tenant_is_refused() {
        let store = Arc::new(MemoryStore::default());
        let (acme, globex) = (tenant("acme"), tenant("globex"));
        tenant_keys(&store)
            .encrypt(&acme, b"totp-secret", b"label")
            .await
            .unwrap();
        let acme_key = store.versions(&acme).remove(0);
        store.rotate_to(&globex, acme_key);

        assert!(
            tenant_keys(&store)
                .encrypt(&globex, b"totp-secret", b"label")
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn unwrapped_keys_are_reused() {
        let store = Arc::new(MemoryStore::default());
        let keys = tenant_keys(&store);
        let acme = tenant("acme");

        let result = keys.encrypt(&acme, b"totp-secret", b"label").await.unwrap();
        let after_first_use = store.calls();
        assert!(after_first_use > 0);

        for _ in 0..5 {
            let result = keys.encrypt(&acme, b"totp-secret", b"label").await.unwrap();
            keys.decrypt(&acme, result.key_version, &result.encrypted, b"label")
                .await
                .unwrap();
        }
        keys.decrypt(&acme, result.key_version, &result.encrypted, b"label")
            .await
            .unwrap();

        assert_eq!(store.calls(), after_first_use);
    }

    /// Requests that need a tenant's first key at once share one.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_first_encryptions_share_one_key() {
        let store = Arc::new(MemoryStore::default());
        let keys = Arc::new(tenant_keys(&store));
        let acme = tenant("acme");

        let encryptions = (0..16)
            .map(|_| {
                let (keys, acme) = (keys.clone(), acme.clone());
                tokio::spawn(
                    async move { keys.encrypt(&acme, b"totp-secret", b"label").await.unwrap() },
                )
            })
            .collect::<Vec<_>>();

        for encryption in encryptions {
            let result = encryption.await.unwrap();
            let plaintext = keys
                .decrypt(&acme, result.key_version, &result.encrypted, b"label")
                .await
                .unwrap();
            assert_eq!(plaintext, b"totp-secret");
        }
        assert_eq!(store.versions(&acme).len(), 1);
    }

    /// After a rotation a server that starts encrypts under the new version
    /// and still decrypts what the old one encrypted.
    #[tokio::test]
    async fn an_older_version_still_decrypts() {
        let store = Arc::new(MemoryStore::default());
        let acme = tenant("acme");
        let before = tenant_keys(&store)
            .encrypt(&acme, b"totp-secret", b"label")
            .await
            .unwrap();
        store.rotate_to(&acme, new_stored_key(&acme, 2).await);

        let keys = tenant_keys(&store);
        let after = keys.encrypt(&acme, b"totp-secret", b"label").await.unwrap();
        assert_eq!((before.key_version, after.key_version), (1, 2));

        for result in [&before, &after] {
            let plaintext = keys
                .decrypt(&acme, result.key_version, &result.encrypted, b"label")
                .await
                .unwrap();
            assert_eq!(plaintext, b"totp-secret");
        }
        // The version says which key: the other one does not decrypt.
        assert!(
            keys.decrypt(&acme, 2, &before.encrypted, b"label")
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn an_unknown_version_is_an_error() {
        let store = Arc::new(MemoryStore::default());
        let keys = tenant_keys(&store);
        let acme = tenant("acme");
        let result = keys.encrypt(&acme, b"totp-secret", b"label").await.unwrap();

        let error = keys
            .decrypt(&acme, 7, &result.encrypted, b"label")
            .await
            .unwrap_err();
        assert!(format!("{error:?}").contains("no key of version 7"));

        // Decrypting never creates a key.
        let globex = tenant("globex");
        assert!(
            keys.decrypt(&globex, 1, &result.encrypted, b"label")
                .await
                .is_err()
        );
        assert!(store.versions(&globex).is_empty());
    }

    #[tokio::test]
    async fn without_a_platform_key_nothing_is_encrypted() {
        let store = Arc::new(MemoryStore::default());
        let keys = TenantKeys::new(store.clone(), None);
        let acme = tenant("acme");

        let error = keys
            .encrypt(&acme, b"totp-secret", b"label")
            .await
            .err()
            .unwrap();
        assert!(format!("{error:?}").contains("No platform key is configured"));
        assert!(store.versions(&acme).is_empty());
        assert_eq!(store.calls(), 0);
    }

    /// A server configured with another platform key says so, rather than
    /// failing to decrypt or creating a second key.
    #[tokio::test]
    async fn a_key_wrapped_by_another_platform_key_is_refused() {
        let store = Arc::new(MemoryStore::default());
        let acme = tenant("acme");
        let result = tenant_keys(&store)
            .encrypt(&acme, b"totp-secret", b"label")
            .await
            .unwrap();

        let misconfigured = TenantKeys::new(store.clone(), Some(wrapper("OTHER_PLATFORM_KEY")));
        for error in [
            misconfigured
                .encrypt(&acme, b"totp-secret", b"label")
                .await
                .err()
                .unwrap(),
            misconfigured
                .decrypt(&acme, result.key_version, &result.encrypted, b"label")
                .await
                .unwrap_err(),
        ] {
            assert!(format!("{error:?}").contains("other than the configured one"));
        }
        assert_eq!(store.versions(&acme).len(), 1);
    }

    /// A failure to load a key is not remembered: the next use tries again.
    #[tokio::test]
    async fn a_failed_load_is_retried() {
        let store = Arc::new(MemoryStore::default());
        let acme = tenant("acme");
        let keys = TenantKeys::new(store.clone(), Some(wrapper("MISSING")));
        assert!(keys.encrypt(&acme, b"totp-secret", b"label").await.is_err());
        assert!(keys.encrypt(&acme, b"totp-secret", b"label").await.is_err());
        assert!(store.versions(&acme).is_empty());
    }
}
