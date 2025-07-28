use crate::lock::{LockKind, LockProvider, postgres::PostgresLockProvider};
use oxidized_config::get_config;
use oxidized_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use sqlx::Connection;

mod lock;

#[derive(OperationOutcomeError, Debug)]
pub enum IndexingWorkerError {
    #[fatal(code = "exception", diagnostic = "Database error: {arg0}")]
    DatabaseConnectionError(#[from] sqlx::Error),
    #[fatal(code = "exception", diagnostic = "Lock error: {arg0}")]
    OperationError(#[from] OperationOutcomeError),
}

#[tokio::main]
pub async fn main() {
    // Initialize the PostgreSQL connection pool
    let config = get_config("environment".into());
    let mut pg_connection = sqlx::PgConnection::connect(&config.get("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to the database");

    loop {
        pg_connection
            .transaction(|t| {
                Box::pin(async move {
                    let mut provider = PostgresLockProvider::new(t);
                    let locks = provider
                        .get_available(LockKind::System, vec!["tenant".into(), "lock2".into()])
                        .await?;

                    println!("Available locks: {:?}", locks);

                    let ret: Result<(), IndexingWorkerError> = Ok(());

                    ret
                })
            })
            .await
            .expect("Transaction failed");
    }
}
