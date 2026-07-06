use haste_fhir_operation_error::derive::OperationOutcomeError;

#[derive(OperationOutcomeError, Debug)]
pub enum EncryptionError {
    #[error(code = "not-found", diagnostic = "Secret '{arg0}' was not found")]
    SecretNotFound(String),

    #[error(
        code = "exception",
        diagnostic = "Failed to retrieve secret '{arg0}': {arg1}"
    )]
    SecretRetrievalFailed(String, String),

    #[error(
        code = "invalid",
        diagnostic = "Encryption key must be {arg0} bytes, got {arg1}"
    )]
    InvalidKeyLength(usize, usize),

    #[error(code = "exception", diagnostic = "Encryption failed: {arg0}")]
    EncryptionFailed(String),

    #[error(code = "exception", diagnostic = "Decryption failed: {arg0}")]
    DecryptionFailed(String),

    #[error(
        code = "not-supported",
        diagnostic = "Secrets provider '{arg0}' is not yet implemented"
    )]
    ProviderNotImplemented(String),
}
