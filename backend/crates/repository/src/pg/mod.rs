use haste_fhir_model::r4::generated::resources::Resource;
use haste_fhir_operation_error::derive::OperationOutcomeError;
use haste_jwt::{AuthorId, AuthorKind, ProjectId, TenantId, VersionId};
use moka::future::Cache;
use sqlx::Postgres;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::Repository;
use crate::types::{FHIRMethod, SupportedFHIRVersions};

mod migrate;
mod models;
mod rate_limit;
mod sequence;
mod transaction;

#[derive(OperationOutcomeError, Debug)]
pub enum StoreError {
    #[error(code = "duplicate", diagnostic = "Resource already exists.")]
    Duplicate,
    #[error(code = "not-found", diagnostic = "Resource not found.")]
    NotFound,
    #[error(code = "invalid", diagnostic = "SQL Error occured.")]
    SQLXError(#[from] sqlx::Error),
    #[error(code = "exception", diagnostic = "Failed to create transaction.")]
    TransactionError,
    #[error(code = "invalid", diagnostic = "Cannot commit non transaction.")]
    NotTransaction,
    #[error(code = "invalid", diagnostic = "Failed to commit the transaction.")]
    FailedCommitTransaction,
    #[error(code = "exception", diagnostic = "Failed to hash password.")]
    PasswordHashError(argon2::password_hash::Error),
}

/// A single buffered `resources` row awaiting a batched multi-row INSERT.
/// Produced by the pure Rust mutation step in `create`/`update`/`delete`
/// (see `pg::models::fhir`), with no DB access involved.
#[derive(Debug, Clone)]
pub struct PendingResourceRow {
    pub tenant: TenantId,
    pub project: ProjectId,
    pub author_id: AuthorId,
    pub author_type: AuthorKind,
    pub fhir_version: SupportedFHIRVersions,
    pub resource: Resource,
    pub deleted: bool,
    pub request_method: &'static str,
    pub fhir_method: FHIRMethod,
}

/// Connection types supported by the repository traits.
#[derive(Debug, Clone)]
pub enum PGConnection {
    Pool(sqlx::Pool<Postgres>, Cache<VersionId, Resource>),
    Transaction(
        Arc<Mutex<sqlx::Transaction<'static, Postgres>>>,
        Cache<VersionId, Resource>,
        Arc<Mutex<Vec<PendingResourceRow>>>,
    ),
}

static TOTAL_CACHE_SIZE: u64 = 1000 * 10;

impl PGConnection {
    #[must_use]
    pub fn pool(pool: sqlx::Pool<Postgres>) -> Self {
        PGConnection::Pool(pool, Cache::new(TOTAL_CACHE_SIZE))
    }

    #[must_use]
    pub fn cache(&self) -> &Cache<VersionId, Resource> {
        match self {
            PGConnection::Pool(_, cache) | PGConnection::Transaction(_, cache, _) => cache,
        }
    }
}

impl Repository for PGConnection {}
