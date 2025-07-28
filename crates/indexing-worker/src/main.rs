use crate::lock::{LockKind, LockProvider, postgres::PostgresLockProvider};
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

    let mut provider = PostgresLockProvider::new(pg_connection);

    let locks = provider
        .get_available(LockKind::System, vec!["tenant".into(), "lock2".into()])
        .await
        .expect("Failed to get available locks");

    println!("Available locks: {:?}", locks);
}
