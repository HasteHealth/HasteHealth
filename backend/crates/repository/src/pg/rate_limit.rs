use crate::pg::{
    PGConnection,
    utilities::{commit_transaction, create_transaction},
};
use haste_rate_limit::{RateLimit, RateLimitError};
use sqlx::{Acquire, Postgres};

fn check_rate_limit<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    rate_key: &'a str,
    max: i32,
    points: i32,
    window_in_seconds: i32,
) -> impl Future<Output = Result<i32, haste_rate_limit::RateLimitError>> + Send + 'a {
    async move {
        let mut conn = connection
            .acquire()
            .await
            .map_err(|_e| RateLimitError::Error("could not acquire connection".to_string()))?;

        let result: i32 = sqlx::query!(
            "SELECT check_rate_limit($1, $2, $3, $4) as current_limit",
            rate_key as &str,
            max as i32,
            points as i32,
            window_in_seconds as i32,
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(|_e| RateLimitError::Exceeded)?
        .current_limit
        .unwrap_or(0);

        Ok(result)
    }
}

impl RateLimit for PGConnection {
    // Returns the current points after the operation.
    async fn check(
        &self,
        rate_key: &str,
        max: i32,
        points: i32,
        window_in_seconds: i32,
    ) -> Result<i32, haste_rate_limit::RateLimitError> {
        match &self {
            PGConnection::Pool(_pool, _) => {
                let tx = create_transaction(self, true)
                    .await
                    .map_err(|e| RateLimitError::Error(e.to_string()))?;
                let res = {
                    let mut conn = tx.lock().await;
                    let res =
                        check_rate_limit(&mut *conn, rate_key, max, points, window_in_seconds)
                            .await?;
                    res
                };
                commit_transaction(tx)
                    .await
                    .map_err(|e| RateLimitError::Error(e.to_string()))?;
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
