use haste_fhir_operation_error::OperationOutcomeError;
use std::{future::Future, pin::Pin};

/// A secret's raw byte value. `Debug` is redacted so the value never
/// ends up in logs or error messages by accident.
pub struct Secret(Vec<u8>);

impl Secret {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

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

/// Symmetric encryption/decryption of arbitrary byte payloads.
pub trait Encryptor: Sync + Send {
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, OperationOutcomeError>;
    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, OperationOutcomeError>;
}
