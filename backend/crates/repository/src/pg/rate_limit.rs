use std::{pin::Pin, sync::LazyLock};

use crate::pg::{
    PGConnection,
    utilities::{commit_transaction, create_transaction},
};
use haste_rate_limit::{RateLimit, RateLimitError};
use moka::future::{Cache, CacheBuilder};
use sqlx::{Acquire, Postgres};

#[derive(Clone)]
enum RateLimitState {
    Count(i32),
    Max,
}

static MEMORY: LazyLock<Cache<String, RateLimitState>> = LazyLock::new(
    // Cache entries live for 30 seconds, after which they will be automatically evicted.
    || {
        CacheBuilder::new(10_000)
            .time_to_idle(std::time::Duration::from_secs(30))
            .build()
    },
);

fn _check_rate_limit_remote<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
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

async fn check_rate_limit_remote<'a>(
    pg: PGConnection,
    rate_key: &'a str,
    max: i32,
    points: i32,
    window_in_seconds: i32,
) -> Result<i32, haste_rate_limit::RateLimitError> {
    match &pg {
        PGConnection::Pool(_pool, _) => {
            let tx = create_transaction(&pg, true)
                .await
                .map_err(|e| RateLimitError::Error(e.to_string()))?;
            let res = {
                let mut conn = tx.lock().await;
                let res =
                    _check_rate_limit_remote(&mut *conn, rate_key, max, points, window_in_seconds)
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
            let res = _check_rate_limit_remote(&mut *tx, rate_key, max, points, window_in_seconds)
                .await?;
            Ok(res)
        }
    }
}

fn check_rate_limit<'a>(
    connection: PGConnection,
    rate_key: &'a str,
    max: i32,
    points: i32,
    window_in_seconds: i32,
) -> impl Future<Output = Result<i32, haste_rate_limit::RateLimitError>> + Send + 'a {
    async move {
        // First check in-memory cache
        if let Some(current) = MEMORY.get(rate_key).await {
            let cloned_key = rate_key.to_string();
            // Run background task to update the cache asynchronously without blocking the main request flow.
            // This allows us to have a fast response time while still keeping the cache reasonably up to date.
            tokio::spawn(async move {
                let result = check_rate_limit_remote(
                    connection,
                    &cloned_key,
                    max,
                    points,
                    window_in_seconds,
                )
                .await;

                if let Ok(points) = result {
                    MEMORY
                        .insert(cloned_key, RateLimitState::Count(points))
                        .await;
                } else if let Err(e) = result {
                    match e {
                        RateLimitError::Exceeded => {
                            // If the rate limit is exceeded, we can set the in-memory cache to max to prevent further requests from hitting the database until the cache expires.
                            MEMORY.insert(cloned_key, RateLimitState::Max).await;
                        }
                        RateLimitError::Error(e) => {
                            println!("Error checking rate limit: {:?}", e);
                        }
                    }
                }
            });

            match current {
                RateLimitState::Count(current) => {
                    let current_score = current + points;

                    if current_score > max {
                        Err(RateLimitError::Exceeded)
                    } else {
                        MEMORY
                            .insert(rate_key.to_string(), RateLimitState::Count(current_score))
                            .await;
                        Ok(current_score)
                    }
                }
                RateLimitState::Max => Err(RateLimitError::Exceeded),
            }
        } else {
            let result =
                check_rate_limit_remote(connection, rate_key, max, points, window_in_seconds)
                    .await?;

            MEMORY
                .insert(rate_key.to_string(), RateLimitState::Count(result))
                .await;

            Ok(result)
        }
    }
}

impl RateLimit for PGConnection {
    /// Returns the current points after the operation.
    /// Note use of box and pin so can satisfy dynamic dispatch requirements.
    fn check<'a>(
        &'a self,
        rate_key: &'a str,
        max: i32,
        points: i32,
        window_in_seconds: i32,
    ) -> Pin<Box<dyn Future<Output = Result<i32, haste_rate_limit::RateLimitError>> + Send + 'a>>
    {
        let connection = self.clone();
        Box::pin(async move {
            let res =
                check_rate_limit(connection, rate_key, max, points, window_in_seconds).await?;
            Ok(res)
        })
    }
}
