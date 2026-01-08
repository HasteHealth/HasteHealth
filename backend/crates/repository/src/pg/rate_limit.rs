use crate::pg::{
    PGConnection,
    utilities::{commit_transaction, create_transaction},
};
use haste_rate_limit::RateLimit;
use sqlx::{Acquire, Postgres};

async fn check_rate_limit<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    rate_key: &str,
    max: i32,
    points: i32,
    window_in_seconds: i32,
) -> Result<(), haste_rate_limit::RateLimitError> {
    // Implement your rate limiting logic here, e.g., querying a database table to track usage.
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
                    check_rate_limit(&mut *conn, rate_key, max, points, window_in_seconds).await?;
                Ok(res)
            }
        }
    }
}
