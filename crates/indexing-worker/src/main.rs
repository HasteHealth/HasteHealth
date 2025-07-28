use crate::lock::{LockProvider, LockType, postgres::PostgresLockProvider};
use oxidized_config::get_config;
use sqlx::Connection;

mod lock;

#[tokio::main]
pub async fn main() {
    // Initialize the PostgreSQL connection pool
    let config = get_config("environment".into());
    let pg_connection = sqlx::PgConnection::connect(&config.get("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to the database");

    let provider = PostgresLockProvider::new(pg_connection);
    provider
        .get_available(
            LockType::IndexingPosition,
            vec!["lock1".into(), "lock2".into()],
        )
        .expect("Failed to get available locks");
}
