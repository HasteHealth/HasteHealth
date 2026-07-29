use std::{pin::Pin, sync::LazyLock};

use crate::pg::{
    PGConnection,
    utilities::{commit_transaction, create_transaction},
};
use haste_rate_limit::{RateLimit, RateLimitError};
use moka::future::{Cache, CacheBuilder};
use sqlx::PgExecutor;

#[derive(Clone, Copy)]
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

async fn check_rate_limit_remote_with_executor<'a, 'e, E>(
    executor: E,
    rate_key: &'a str,
    max: i32,
    points: i32,
    window_in_seconds: i32,
) -> Result<i32, haste_rate_limit::RateLimitError>
where
    E: PgExecutor<'e>,
{
    let result =
        sqlx::query_as::<_, (i32,)>("SELECT check_rate_limit($1, $2, $3, $4) as current_limit")
            .bind(rate_key)
            .bind(max)
            .bind(points)
            .bind(window_in_seconds)
            .fetch_one(executor)
            .await
            .map_err(|_e| RateLimitError::Exceeded)?;

    Ok(result.0)
}

async fn check_rate_limit_remote(
    pg: PGConnection,
    rate_key: &str,
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
                check_rate_limit_remote_with_executor(
                    &mut **conn,
                    rate_key,
                    max,
                    points,
                    window_in_seconds,
                )
                .await?
            };
            commit_transaction(tx)
                .await
                .map_err(|e| RateLimitError::Error(e.to_string()))?;
            Ok(res)
        }
        PGConnection::Transaction(tx, _) => {
            let mut tx = tx.lock().await;
            check_rate_limit_remote_with_executor(
                &mut **tx,
                rate_key,
                max,
                points,
                window_in_seconds,
            )
            .await
        }
    }
}

async fn check_rate_limit(
    connection: PGConnection,
    rate_key: &str,
    max: i32,
    points: i32,
    window_in_seconds: i32,
) -> Result<i32, haste_rate_limit::RateLimitError> {
    if let Some(current) = MEMORY.get(rate_key).await {
        let cloned_key = rate_key.to_string();
        let connection_clone = connection.clone();

        tokio::spawn(async move {
            let result = check_rate_limit_remote(
                connection_clone,
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
                        MEMORY.insert(cloned_key, RateLimitState::Max).await;
                    }
                    RateLimitError::Error(e) => {
                        println!("Error checking rate limit: {e:?}");
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
            check_rate_limit_remote(connection, rate_key, max, points, window_in_seconds).await?;

        MEMORY
            .insert(rate_key.to_string(), RateLimitState::Count(result))
            .await;

        Ok(result)
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
