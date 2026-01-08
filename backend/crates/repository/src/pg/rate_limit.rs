use crate::pg::{
    PGConnection,
    utilities::{commit_transaction, create_transaction},
};
use haste_rate_limit::{RateLimit, RateLimitError};
use sqlx::{Acquire, Postgres};
use sqlx_postgres::PgRow;

async fn check_rate_limit<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    rate_key: &str,
    max: i32,
    points: i32,
    window_in_seconds: i32,
) -> Result<(), haste_rate_limit::RateLimitError> {
    let result: PgRow = sqlx::query!(
        r#"select check_rate_limit($1, $2, $3, $4)"#,
        rate_key as &str,
        max as i32,
        points as i32,
        window_in_seconds as i32,
    )
    .fetch_one(&mut *connection)
    .await
    .map_err(|_e| RateLimitError::Exceeded)?;

    Ok(())
}

impl RateLimit for PGConnection {
    async fn check(
        &self,
        rate_key: &str,
        max: i32,
        points: i32,
        window_in_seconds: i32,
    ) -> Result<(), haste_rate_limit::RateLimitError> {
        match &self {
            PGConnection::Pool(_pool, _) => {
                let tx = create_transaction(self, true).await?;
                let res = {
                    let mut conn = tx.lock().await;
                    let res =
                        check_rate_limit(&mut *conn, rate_key, max, points, window_in_seconds)
                            .await?;
                    res
                };
                commit_transaction(tx).await?;
                Ok(res)
            }
            PGConnection::Transaction(tx, _) => {
                let mut tx = tx.lock().await;
                let res =
                    check_rate_limit(&mut *tx, rate_key, max, points, window_in_seconds).await?;
                Ok(res)
            }
        }
    }
}
