use sqlx::{Pool, Postgres};
use sqlx_postgres::PgPoolOptions;
use tokio::sync::OnceCell;
use tracing::info;

// Singleton for the database connection pool in postgres.
static POOL: OnceCell<Pool<Postgres>> = OnceCell::const_new();
pub async fn get_pool() -> &'static Pool<Postgres> {
    POOL.get_or_init(async || {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        info!("Connecting to postgres database");
        let connection = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to create database connection pool");
        connection
    })
    .await
}
