use std::fmt::{self, Display};

use haste_fhir_operation_error::derive::OperationOutcomeError;

/// Failure of a [`DocumentStorage`](crate::DocumentStorage) call.
#[derive(OperationOutcomeError, Debug)]
pub enum StorageError {
    /// The key breaks the rules documented on [`DocumentKey`](crate::DocumentKey).
    #[error(code = "invalid", diagnostic = "Invalid document key: {arg0}")]
    InvalidKey(String),

    /// The document cannot be stored as given, for example because its
    /// content type is not a valid header value.
    #[error(code = "invalid", diagnostic = "Invalid document: {arg0}")]
    InvalidDocument(String),

    /// The backend could not be set up from its configuration or credentials.
    #[error(
        code = "exception",
        diagnostic = "Document storage is misconfigured: {arg0}"
    )]
    Configuration(String),

    /// The backend rejected the server's credentials.
    #[error(
        code = "exception",
        diagnostic = "Document storage rejected the credentials: {arg0}"
    )]
    Unauthorized(String),

    /// The credentials are valid but lack permission for the operation.
    #[error(
        code = "exception",
        diagnostic = "Document storage denied access: {arg0}"
    )]
    Forbidden(String),

    /// Nothing is stored under the key (or the bucket itself is missing).
    #[error(code = "not-found", diagnostic = "Document '{arg0}' was not found")]
    NotFound(String),

    /// The request never completed: connection, timeout or body transfer failure.
    #[error(
        code = "transient",
        diagnostic = "Document storage request failed: {arg0}"
    )]
    Transport(String),

    /// The backend answered with an unexpected status or body.
    #[error(
        code = "exception",
        diagnostic = "Document storage responded with status {arg0}: {arg1}"
    )]
    Backend(u16, String),
}

impl StorageError {
    /// Whether retrying the same request may succeed.
    #[must_use]
    pub fn is_transient(&self) -> bool {
        match self {
            Self::Transport(_) => true,
            Self::Backend(status, _) => matches!(status, 408 | 429 | 500 | 502 | 503 | 504),
            _ => false,
        }
    }
}

impl Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidKey(reason) => write!(f, "invalid document key: {reason}"),
            Self::InvalidDocument(reason) => write!(f, "invalid document: {reason}"),
            Self::Configuration(reason) => write!(f, "document storage misconfigured: {reason}"),
            Self::Unauthorized(reason) => write!(f, "credentials rejected: {reason}"),
            Self::Forbidden(reason) => write!(f, "access denied: {reason}"),
            Self::NotFound(key) => write!(f, "document '{key}' not found"),
            Self::Transport(reason) => write!(f, "request failed: {reason}"),
            Self::Backend(status, reason) => write!(f, "backend responded {status}: {reason}"),
        }
    }
}

impl std::error::Error for StorageError {}
