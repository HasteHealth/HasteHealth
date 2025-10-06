use oxidized_fhir_operation_error::derive::OperationOutcomeError;
use sqlx::Postgres;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::Repository;

mod authorization_code;
mod fhir;
mod membership;
mod tenant;
mod user;

#[derive(OperationOutcomeError, Debug)]
pub enum StoreError {
    #[error(code = "invalid", diagnostic = "SQL Error occured.")]
    SQLXError(#[from] sqlx::Error),
    #[error(code = "exception", diagnostic = "Failed to create transaction.")]
    TransactionError,
    #[error(code = "invalid", diagnostic = "Cannot commit non transaction.")]
    NotTransaction,
    #[error(code = "invalid", diagnostic = "Failed to commit the transaction.")]
    FailedCommitTransaction,
}

/// Connection types supported by the repository traits.
#[derive(Debug, Clone)]
pub enum PGConnection {
    PgPool(sqlx::Pool<Postgres>),
    PgTransaction(Arc<Mutex<sqlx::Transaction<'static, Postgres>>>),
}

impl Repository for PGConnection {}
