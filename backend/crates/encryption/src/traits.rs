use haste_fhir_operation_error::OperationOutcomeError;
use std::{future::Future, pin::Pin};

/// A secret's raw byte value. `Debug` is redacted so the value never
/// ends up in logs or error messages by accident.
pub struct Secret(Vec<u8>);

impl Secret {
    #[must_use]
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub fn expose_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Secret(REDACTED)")
    }
}

/// Retrieves secret material (encryption keys, credentials, etc.) by name
/// from a backing store, e.g. AWS Secrets Manager, GCP Secret Manager, or
/// environment variables.
pub trait SecretsProvider: Sync + Send {
    fn get_secret<'a>(
        &'a self,
        name: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Secret, OperationOutcomeError>> + Send + 'a>>;
}

pub struct EncryptionResult {
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
}

/// Provides symmetric authenticated encryption and decryption of arbitrary
/// byte payloads.
///
/// Implementations are expected to encrypt data in a way that ensures both
/// confidentiality and integrity. A value returned by [`Self::encrypt`]
/// should be decryptable only by the same implementation initialized with the
/// same keying material.
pub trait Encryptor: Sync + Send {
    /// Encrypts the given plaintext: [`Self::encrypt_with_aad`] with empty
    /// associated data.
    ///
    /// # Errors
    ///
    /// Returns an [`OperationOutcomeError`] if the plaintext cannot be
    /// encrypted.
    fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptionResult, OperationOutcomeError> {
        self.encrypt_with_aad(plaintext, &[])
    }
    /// Decrypts a payload encrypted with empty associated data.
    ///
    /// # Errors
    ///
    /// Returns an [`OperationOutcomeError`] if the ciphertext is invalid,
    /// has been tampered with, or cannot be decrypted.
    fn decrypt(&self, ciphertext: &EncryptionResult) -> Result<Vec<u8>, OperationOutcomeError> {
        self.decrypt_with_aad(ciphertext, &[])
    }
    /// Encrypts the given plaintext and binds the result to `aad`, its
    /// associated data: what the value belongs to, such as the key of the row
    /// that stores it. The associated data is neither secret nor part of the
    /// result. Decryption needs the same bytes again, so a result copied to
    /// another owner does not decrypt there.
    ///
    /// # Errors
    ///
    /// Returns an [`OperationOutcomeError`] if the plaintext cannot be
    /// encrypted.
    fn encrypt_with_aad(
        &self,
        plaintext: &[u8],
        aad: &[u8],
    ) -> Result<EncryptionResult, OperationOutcomeError>;
    /// Decrypts a payload encrypted with the associated data `aad`.
    ///
    /// # Errors
    ///
    /// Returns an [`OperationOutcomeError`] if the ciphertext is invalid,
    /// has been tampered with, was encrypted with other associated data, or
    /// cannot be decrypted.
    fn decrypt_with_aad(
        &self,
        ciphertext: &EncryptionResult,
        aad: &[u8],
    ) -> Result<Vec<u8>, OperationOutcomeError>;
}

/// Wraps a data key: encrypts it with a platform key, so the data key can be
/// stored next to what it encrypts. The platform key stays with the
/// implementation, e.g. inside a cloud KMS, and is never handed out.
pub trait KeyWrapper: Sync + Send {
    /// Names the platform key that wraps. It is stored with each wrapped key,
    /// to tell which platform key unwraps it.
    fn key_id(&self) -> &str;

    /// Wraps `key` and binds the result to `context`: what the key belongs
    /// to, such as a tenant id. The context is not secret and is not part of
    /// the result; unwrapping needs the same context again.
    fn wrap_key<'a>(
        &'a self,
        key: &'a [u8],
        context: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, OperationOutcomeError>> + Send + 'a>>;

    /// Unwraps what [`Self::wrap_key`] returned for `context`. Fails when
    /// `wrapped_key` was wrapped for another context or by another platform
    /// key, or was altered.
    fn unwrap_key<'a>(
        &'a self,
        wrapped_key: &'a [u8],
        context: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Secret, OperationOutcomeError>> + Send + 'a>>;
}
