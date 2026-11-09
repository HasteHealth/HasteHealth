use std::fmt::{self, Display};

use haste_fhir_operation_error::derive::OperationOutcomeError;

/// Failure of a [`DocumentStorage`](crate::DocumentStorage) call.
#[derive(OperationOutcomeError, Debug)]
pub enum StorageError {
    /// A tenant or project id cannot serve as a path segment; see
    /// [`DocumentScope::new`](crate::DocumentScope::new).
    #[error(code = "invalid", diagnostic = "Invalid document scope: {arg0}")]
    InvalidScope(String),

    /// The name breaks the rules documented on
    /// [`DocumentName`](crate::DocumentName).
    #[error(code = "invalid", diagnostic = "Invalid document name: {arg0}")]
    InvalidName(String),

    /// A listing was continued with a token the backend did not issue.
    #[error(code = "invalid", diagnostic = "Invalid page token: {arg0}")]
    InvalidPageToken(String),

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

    /// The scope holds no document of that name (or the bucket itself is
    /// missing).
    #[error(code = "not-found", diagnostic = "Document '{arg0}' was not found")]
    NotFound(String),

    /// The request never completed: connection, timeout or body transfer failure.
    #[error(
        code = "transient",
        diagnostic = "Document storage request failed: {arg0}"
    )]
    Transport(String),

    /// A filesystem operation failed for a reason other than a missing file
    /// or a denied permission.
    #[error(code = "exception", diagnostic = "Document storage I/O failed: {arg0}")]
    Io(String),

    /// The database behind the Postgres backend rejected a statement.
    #[error(
        code = "exception",
        diagnostic = "Document storage database error: {arg0}"
    )]
    Database(String),

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
            Self::InvalidScope(reason) => write!(f, "invalid document scope: {reason}"),
            Self::InvalidName(reason) => write!(f, "invalid document name: {reason}"),
            Self::InvalidPageToken(reason) => write!(f, "invalid page token: {reason}"),
            Self::InvalidDocument(reason) => write!(f, "invalid document: {reason}"),
            Self::Configuration(reason) => write!(f, "document storage misconfigured: {reason}"),
            Self::Unauthorized(reason) => write!(f, "credentials rejected: {reason}"),
            Self::Forbidden(reason) => write!(f, "access denied: {reason}"),
            Self::NotFound(key) => write!(f, "document '{key}' not found"),
            Self::Transport(reason) => write!(f, "request failed: {reason}"),
            Self::Io(reason) => write!(f, "I/O failed: {reason}"),
            Self::Database(reason) => write!(f, "database error: {reason}"),
            Self::Backend(status, reason) => write!(f, "backend responded {status}: {reason}"),
        }
    }
}

impl std::error::Error for StorageError {}
