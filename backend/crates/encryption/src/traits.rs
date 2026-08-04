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
    /// Encrypts the given plaintext.
    ///
    /// # Errors
    ///
    /// Returns an [`OperationOutcomeError`] if the plaintext cannot be
    /// encrypted.
    fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptionResult, OperationOutcomeError>;
    /// Decrypts a previously encrypted payload.
    ///
    /// # Errors
    ///
    /// Returns an [`OperationOutcomeError`] if the ciphertext is invalid,
    /// has been tampered with, or cannot be decrypted.
    fn decrypt(&self, ciphertext: &EncryptionResult) -> Result<Vec<u8>, OperationOutcomeError>;
}
