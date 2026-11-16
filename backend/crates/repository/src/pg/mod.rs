use haste_fhir_operation_error::derive::OperationOutcomeError;
use sqlx::Postgres;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::Repository;

pub use pending::PendingRows;
pub use version_cache::VersionCache;

mod async_job;
mod binary_data;
mod client_assertion;
mod failed_indexing;
mod migrate;
mod models;
mod pending;
mod rate_limit;
mod sequence;
mod transaction;
mod version_cache;

#[derive(OperationOutcomeError, Debug)]
pub enum StoreError {
    #[error(code = "duplicate", diagnostic = "Resource already exists.")]
    Duplicate,
    #[error(code = "not-found", diagnostic = "Resource not found.")]
    NotFound,
    // A failure of the database, not of the request: answers 500.
    #[error(code = "exception", diagnostic = "Database operation failed.")]
    SQLXError(#[from] sqlx::Error),
    #[error(code = "exception", diagnostic = "Failed to create transaction.")]
    TransactionError,
    #[error(code = "invalid", diagnostic = "Cannot commit non transaction.")]
    NotTransaction,
    #[error(code = "invalid", diagnostic = "Failed to commit the transaction.")]
    FailedCommitTransaction,
    #[error(code = "exception", diagnostic = "Failed to hash password.")]
    PasswordHashError(argon2::password_hash::Error),
    #[error(
        code = "exception",
        diagnostic = "Failed to deserialize resource: '{arg0}'"
    )]
    DeserializeError(String),
}

/// Connection types supported by the repository traits.
#[derive(Debug, Clone)]
pub enum PGConnection {
    Pool(sqlx::Pool<Postgres>, VersionCache),
    Transaction(
        Arc<Mutex<sqlx::Transaction<'static, Postgres>>>,
        VersionCache,
        PendingRows,
    ),
}

static TOTAL_CACHE_SIZE: u64 = 1000 * 10;

impl PGConnection {
    #[must_use]
    pub fn pool(pool: sqlx::Pool<Postgres>) -> Self {
        PGConnection::Pool(pool, VersionCache::new(TOTAL_CACHE_SIZE))
    }

    #[must_use]
    pub fn cache(&self) -> &VersionCache {
        match self {
            PGConnection::Pool(_, cache) | PGConnection::Transaction(_, cache, _) => cache,
        }
    }
}

impl Repository for PGConnection {}
