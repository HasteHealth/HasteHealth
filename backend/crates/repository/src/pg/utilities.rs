use crate::pg::{PGConnection, StoreError};
use haste_fhir_operation_error::OperationOutcomeError;
use sqlx::{Postgres, Transaction};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn create_transaction(
    connection: &PGConnection,
    is_updating_sequence: bool,
) -> Result<Arc<Mutex<Transaction<'static, Postgres>>>, OperationOutcomeError> {
    match connection {
        PGConnection::Pool(pool, _cache) => {
            let tx = if is_updating_sequence {
                pool.begin_with(
                    "BEGIN; SELECT register_sequence_transaction('resources_sequence_seq')",
                )
                .await
                .map_err(StoreError::from)?
            } else {
                pool.begin().await.map_err(StoreError::from)?
            };

            Ok(Arc::new(Mutex::new(tx)))
        }
        PGConnection::Transaction(tx, _) => Ok(tx.clone()), // Transaction doesn't live long enough so cannot create.
    }
}

pub async fn commit_transaction(
    tx: Arc<Mutex<Transaction<'static, Postgres>>>,
) -> Result<(), OperationOutcomeError> {
    let conn = Mutex::into_inner(Arc::try_unwrap(tx).map_err(|e| {
        println!("Error during commit: {:?}", e);
        StoreError::FailedCommitTransaction
    })?);

    // Handle PgConnection connection
    let res = conn.commit().await.map_err(StoreError::from)?;
    Ok(res)
}

#[allow(dead_code)]
pub async fn rollback_transaction(
    tx: Arc<Mutex<Transaction<'static, Postgres>>>,
) -> Result<(), OperationOutcomeError> {
    let conn = Mutex::into_inner(Arc::try_unwrap(tx).map_err(|e| {
        println!("Error during rollback: {:?}", e);
        StoreError::FailedCommitTransaction
    })?);

    // Handle PgConnection connection
    let res = conn.rollback().await.map_err(StoreError::from)?;
    Ok(res)
}
