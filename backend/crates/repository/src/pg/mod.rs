use haste_fhir_model::r4::generated::resources::Resource;
use haste_fhir_operation_error::derive::OperationOutcomeError;
use haste_jwt::VersionId;
use moka::future::Cache;
use sqlx::Postgres;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{Repository, binary_storage::BinaryStorage};

pub use binary_documents::SweepReport;
pub use pending::PendingRows;

mod async_job;
mod binary_documents;
mod failed_indexing;
mod migrate;
mod models;
mod pending;
mod rate_limit;
mod sequence;
mod transaction;

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
    Pool(sqlx::Pool<Postgres>, Shared),
    Transaction(
        Arc<Mutex<sqlx::Transaction<'static, Postgres>>>,
        Shared,
        PendingRows,
    ),
}

/// What a pool and every transaction opened from it share.
#[derive(Debug, Clone)]
pub struct Shared {
    /// Resources by version. Binary content kept in document storage is not
    /// cached: entries hold the database row's form.
    cache: Cache<VersionId, Resource>,
    binary_storage: Option<BinaryStorage>,
}

static TOTAL_CACHE_SIZE: u64 = 1000 * 10;

impl PGConnection {
    #[must_use]
    pub fn pool(pool: sqlx::Pool<Postgres>) -> Self {
        PGConnection::Pool(
            pool,
            Shared {
                cache: Cache::new(TOTAL_CACHE_SIZE),
                binary_storage: None,
            },
        )
    }

    /// Keeps large `Binary.data` in `storage` rather than in the database.
    #[must_use]
    pub fn with_binary_storage(mut self, storage: BinaryStorage) -> Self {
        self.shared_mut().binary_storage = Some(storage);
        self
    }

    fn shared(&self) -> &Shared {
        match self {
            PGConnection::Pool(_, shared) | PGConnection::Transaction(_, shared, _) => shared,
        }
    }

    fn shared_mut(&mut self) -> &mut Shared {
        match self {
            PGConnection::Pool(_, shared) | PGConnection::Transaction(_, shared, _) => shared,
        }
    }

    #[must_use]
    pub fn cache(&self) -> &Cache<VersionId, Resource> {
        &self.shared().cache
    }

    #[must_use]
    pub fn binary_storage(&self) -> Option<&BinaryStorage> {
        self.shared().binary_storage.as_ref()
    }
}

impl Repository for PGConnection {}
