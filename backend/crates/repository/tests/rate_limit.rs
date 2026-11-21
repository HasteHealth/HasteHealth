//! Integration tests for `check_rate_limit_window`
//! (see `pg-migrations/20261111120000_rate_limit_window_reset.sql`).
//!
//! Requires a reachable Postgres instance, e.g. `docker-services-compose.yml`'s
//! `postgres` service, with `DATABASE_URL` set to an admin-capable connection
//! string (`sqlx::test` creates and drops an ephemeral database per test):
//!
//! ```sh
//! DATABASE_URL=postgresql://postgres:postgres@127.0.0.1/haste_health cargo test -p haste-repository
//! ```

use sqlx::PgPool;

async fn check(
    pool: &PgPool,
    key: &str,
    max: i32,
    points: i32,
    window_seconds: i32,
) -> sqlx::Result<(i32, i64, bool)> {
    sqlx::query_as(
        "SELECT spent, resets_in_ms, exceeded FROM check_rate_limit_window($1, $2, $3, $4)",
    )
    .bind(key)
    .bind(max)
    .bind(points)
    .bind(window_seconds)
    .fetch_one(pool)
    .await
}

/// Points add up within a window, and each answer says how long the window
/// has left.
#[sqlx::test(migrations = "./pg-migrations")]
async fn spends_points_and_reports_the_reset(pool: PgPool) -> sqlx::Result<()> {
    let (spent, resets_in_ms, exceeded) = check(&pool, "tenant", 100, 40, 900).await?;
    assert_eq!((spent, exceeded), (40, false));
    assert!(
        (899_000..=900_000).contains(&resets_in_ms),
        "a new window resets a full window from now, not {resets_in_ms}ms"
    );

    let (spent, _, exceeded) = check(&pool, "tenant", 100, 60, 900).await?;
    assert_eq!((spent, exceeded), (100, false));
    Ok(())
}

/// A refusal spends nothing, so a smaller request that still fits succeeds
/// after a bigger one is refused, and the refusal says when to retry.
#[sqlx::test(migrations = "./pg-migrations")]
async fn refuses_without_spending(pool: PgPool) -> sqlx::Result<()> {
    check(&pool, "tenant", 100, 90, 900).await?;

    let (spent, resets_in_ms, exceeded) = check(&pool, "tenant", 100, 25, 900).await?;
    assert_eq!((spent, exceeded), (90, true));
    assert!(resets_in_ms > 0 && resets_in_ms <= 900_000);

    let (spent, _, exceeded) = check(&pool, "tenant", 100, 10, 900).await?;
    assert_eq!((spent, exceeded), (100, false));
    Ok(())
}

/// Once the window has passed, the count starts again from the new request.
#[sqlx::test(migrations = "./pg-migrations")]
async fn starts_a_new_window_once_the_old_one_ends(pool: PgPool) -> sqlx::Result<()> {
    check(&pool, "tenant", 100, 100, 900).await?;
    let (_, _, exceeded) = check(&pool, "tenant", 100, 1, 900).await?;
    assert!(exceeded);

    sqlx::query("UPDATE rate_limits SET window_start = window_start - interval '901 seconds'")
        .execute(&pool)
        .await?;

    let (spent, resets_in_ms, exceeded) = check(&pool, "tenant", 100, 1, 900).await?;
    assert_eq!((spent, exceeded), (1, false));
    assert!(resets_in_ms > 899_000);
    Ok(())
}

/// Keys are counted independently.
#[sqlx::test(migrations = "./pg-migrations")]
async fn counts_each_key_separately(pool: PgPool) -> sqlx::Result<()> {
    check(&pool, "tenant-a", 100, 100, 900).await?;

    let (spent, _, exceeded) = check(&pool, "tenant-b", 100, 100, 900).await?;
    assert_eq!((spent, exceeded), (100, false));
    Ok(())
}
