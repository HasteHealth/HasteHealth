use std::{
    pin::Pin,
    sync::LazyLock,
    time::{Duration, Instant},
};

use crate::pg::{
    PGConnection,
    transaction::{commit_transaction, create_transaction},
};
use haste_rate_limit::{RateLimit, RateLimitError};
use moka::future::{Cache, CacheBuilder};
use sqlx::PgExecutor;

/// The points spent in a key's current window, and when that window ends.
#[derive(Clone, Copy)]
struct RateLimitState {
    points: i32,
    resets_at: Instant,
}

impl RateLimitState {
    fn exceeded(&self) -> RateLimitError {
        RateLimitError::Exceeded {
            retry_after: self.resets_at.saturating_duration_since(Instant::now()),
        }
    }
}

/// What one call to `check_rate_limit_window` found.
struct RemoteCheck {
    state: RateLimitState,
    exceeded: bool,
}

impl RemoteCheck {
    fn into_result(self) -> Result<i32, RateLimitError> {
        if self.exceeded {
            Err(self.state.exceeded())
        } else {
            Ok(self.state.points)
        }
    }
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
) -> Result<RemoteCheck, String>
where
    E: PgExecutor<'e>,
{
    let (spent, resets_in_ms, exceeded) = sqlx::query_as::<_, (i32, i64, bool)>(
        "SELECT spent, resets_in_ms, exceeded FROM check_rate_limit_window($1, $2, $3, $4)",
    )
    .bind(rate_key)
    .bind(max)
    .bind(points)
    .bind(window_in_seconds)
    .fetch_one(executor)
    .await
    .map_err(|e| e.to_string())?;

    // Postgres measures the time left on its own clock, so the reset time
    // does not depend on the two clocks agreeing.
    let resets_in = Duration::from_millis(u64::try_from(resets_in_ms).unwrap_or(0));

    Ok(RemoteCheck {
        state: RateLimitState {
            points: spent,
            resets_at: Instant::now() + resets_in,
        },
        exceeded,
    })
}

async fn check_rate_limit_remote(
    pg: PGConnection,
    rate_key: &str,
    max: i32,
    points: i32,
    window_in_seconds: i32,
) -> Result<RemoteCheck, String> {
    match &pg {
        PGConnection::Pool(_pool, _) => {
            let tx = create_transaction(&pg, false)
                .await
                .map_err(|e| e.to_string())?;
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
            commit_transaction(tx).await.map_err(|e| e.to_string())?;
            Ok(res)
        }
        PGConnection::Transaction(tx, _, _) => {
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
    // A cached window that has already ended says nothing about the new one,
    // so ask Postgres instead of refusing on a stale count.
    let cached = MEMORY
        .get(rate_key)
        .await
        .filter(|state| state.resets_at > Instant::now());

    if let Some(current) = cached {
        let cloned_key = rate_key.to_string();
        let connection_clone = connection.clone();

        tokio::spawn(async move {
            match check_rate_limit_remote(
                connection_clone,
                &cloned_key,
                max,
                points,
                window_in_seconds,
            )
            .await
            {
                Ok(remote) => MEMORY.insert(cloned_key, remote.state).await,
                Err(e) => println!("Error checking rate limit: {e:?}"),
            }
        });

        let current_score = current.points + points;

        if current_score > max {
            Err(current.exceeded())
        } else {
            MEMORY
                .insert(
                    rate_key.to_string(),
                    RateLimitState {
                        points: current_score,
                        resets_at: current.resets_at,
                    },
                )
                .await;
            Ok(current_score)
        }
    } else {
        let remote = check_rate_limit_remote(connection, rate_key, max, points, window_in_seconds)
            .await
            .map_err(RateLimitError::Error)?;

        MEMORY.insert(rate_key.to_string(), remote.state).await;

        remote.into_result()
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

    /// Goes straight to Postgres, skipping [`MEMORY`].
    /// `check_rate_limit_window` takes a per-key advisory transaction lock, so
    /// concurrent callers are counted one at a time with no lost increments.
    /// The in-memory cache is what makes [`RateLimit::check`] fast but
    /// imprecise under a burst; this trades that for an exact count on every
    /// call.
    fn check_uncached<'a>(
        &'a self,
        rate_key: &'a str,
        max: i32,
        points: i32,
        window_in_seconds: i32,
    ) -> Pin<Box<dyn Future<Output = Result<i32, haste_rate_limit::RateLimitError>> + Send + 'a>>
    {
        let connection = self.clone();
        Box::pin(async move {
            check_rate_limit_remote(connection, rate_key, max, points, window_in_seconds)
                .await
                .map_err(RateLimitError::Error)?
                .into_result()
        })
    }
}
