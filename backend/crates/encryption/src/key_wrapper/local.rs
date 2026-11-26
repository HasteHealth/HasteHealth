use crate::{
    encryption::aes::AesGcmEncryptor,
    error::EncryptionError,
    traits::{EncryptionResult, Encryptor as _, KeyWrapper, Secret, SecretsProvider},
};
use haste_fhir_operation_error::OperationOutcomeError;
use std::{future::Future, pin::Pin, sync::Arc};

/// Length of the AES-GCM nonce a wrapped key starts with.
const NONCE_LEN: usize = 12;

/// Wraps with an AES-256-GCM platform key this process reads from its
/// secrets provider. For development and for deployments without a KMS: the
/// platform key is in the server's memory while it wraps.
///
/// A wrapped key is `nonce || ciphertext`.
pub struct LocalKeyWrapper {
    secrets: Arc<dyn SecretsProvider>,
    secret_name: String,
}

impl LocalKeyWrapper {
    /// `secret_name` is the name `secrets` holds the platform key under. It
    /// is also this wrapper's [`KeyWrapper::key_id`], so a new platform key
    /// goes under a new name.
    #[must_use]
    pub fn new(secrets: Arc<dyn SecretsProvider>, secret_name: String) -> Self {
        Self {
            secrets,
            secret_name,
        }
    }

    async fn platform_key(&self) -> Result<AesGcmEncryptor, OperationOutcomeError> {
        let platform_key = self.secrets.get_secret(&self.secret_name).await?;
        AesGcmEncryptor::new(platform_key.expose_bytes())
    }
}

impl KeyWrapper for LocalKeyWrapper {
    fn key_id(&self) -> &str {
        &self.secret_name
    }

    fn wrap_key<'a>(
        &'a self,
        key: &'a [u8],
        context: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, OperationOutcomeError>> + Send + 'a>> {
        Box::pin(async move {
            let EncryptionResult {
                mut nonce,
                ciphertext,
            } = self
                .platform_key()
                .await?
                .encrypt_with_aad(key, context.as_bytes())?;

            nonce.extend_from_slice(&ciphertext);
            Ok(nonce)
        })
    }

    fn unwrap_key<'a>(
        &'a self,
        wrapped_key: &'a [u8],
        context: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Secret, OperationOutcomeError>> + Send + 'a>> {
        Box::pin(async move {
            let Some((nonce, ciphertext)) = wrapped_key.split_at_checked(NONCE_LEN) else {
                return Err(EncryptionError::KeyUnwrapFailed(
                    "the wrapped key is too short".to_string(),
                )
                .into());
            };

            let key = self
                .platform_key()
                .await?
                .decrypt_with_aad(
                    &EncryptionResult {
                        nonce: nonce.to_vec(),
                        ciphertext: ciphertext.to_vec(),
                    },
                    context.as_bytes(),
                )
                .map_err(|_| {
                    EncryptionError::KeyUnwrapFailed(
                        "it was wrapped for another context or by another platform key, or was altered"
                            .to_string(),
                    )
                })?;

            Ok(Secret::new(key))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Platform keys by name, in memory.
    struct StaticSecrets(HashMap<&'static str, [u8; 32]>);

    impl SecretsProvider for StaticSecrets {
        fn get_secret<'a>(
            &'a self,
            name: &'a str,
        ) -> Pin<Box<dyn Future<Output = Result<Secret, OperationOutcomeError>> + Send + 'a>>
        {
            Box::pin(async move {
                self.0
                    .get(name)
                    .map(|key| Secret::new(key.to_vec()))
                    .ok_or_else(|| EncryptionError::SecretNotFound(name.to_string()).into())
            })
        }
    }

    fn wrapper(secret_name: &str) -> LocalKeyWrapper {
        LocalKeyWrapper::new(
            Arc::new(StaticSecrets(HashMap::from([
                ("PLATFORM_KEY", [1u8; 32]),
                ("OTHER_PLATFORM_KEY", [2u8; 32]),
            ]))),
            secret_name.to_string(),
        )
    }

    /// Drives a future that never waits, as these do.
    fn now<T>(future: impl Future<Output = T>) -> T {
        let mut future = std::pin::pin!(future);
        let mut context = std::task::Context::from_waker(std::task::Waker::noop());
        match future.as_mut().poll(&mut context) {
            std::task::Poll::Ready(value) => value,
            std::task::Poll::Pending => panic!("the future waited"),
        }
    }

    #[test]
    fn round_trips_a_key() {
        let wrapper = wrapper("PLATFORM_KEY");
        let key = [9u8; 32];

        let wrapped = now(wrapper.wrap_key(&key, "tenant-a")).unwrap();
        assert!(!wrapped.windows(key.len()).any(|window| window == key));

        let unwrapped = now(wrapper.unwrap_key(&wrapped, "tenant-a")).unwrap();
        assert_eq!(unwrapped.expose_bytes(), key);
    }

    #[test]
    fn is_named_after_its_platform_key() {
        assert_eq!(wrapper("PLATFORM_KEY").key_id(), "PLATFORM_KEY");
    }

    /// A wrapped key copied to another tenant's row does not unwrap there.
    #[test]
    fn rejects_another_context() {
        let wrapper = wrapper("PLATFORM_KEY");
        let wrapped = now(wrapper.wrap_key(&[9u8; 32], "tenant-a")).unwrap();

        assert!(now(wrapper.unwrap_key(&wrapped, "tenant-b")).is_err());
    }

    #[test]
    fn rejects_another_platform_key() {
        let wrapped = now(wrapper("PLATFORM_KEY").wrap_key(&[9u8; 32], "tenant-a")).unwrap();

        assert!(now(wrapper("OTHER_PLATFORM_KEY").unwrap_key(&wrapped, "tenant-a")).is_err());
    }

    #[test]
    fn rejects_an_altered_or_short_wrapped_key() {
        let wrapper = wrapper("PLATFORM_KEY");
        let mut wrapped = now(wrapper.wrap_key(&[9u8; 32], "tenant-a")).unwrap();

        assert!(now(wrapper.unwrap_key(&wrapped[..NONCE_LEN - 1], "tenant-a")).is_err());

        let last = wrapped.len() - 1;
        wrapped[last] ^= 0xFF;
        assert!(now(wrapper.unwrap_key(&wrapped, "tenant-a")).is_err());
    }

    #[test]
    fn fails_without_its_platform_key() {
        assert!(now(wrapper("MISSING").wrap_key(&[9u8; 32], "tenant-a")).is_err());
    }
}
