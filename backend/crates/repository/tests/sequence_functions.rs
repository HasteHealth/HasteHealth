//! Integration tests for `register_sequence_transaction` and `max_safe_seq`
//! (see `pg-migrations/20260911202548_sequence_fn_improvements.sql`).
//!
//! Requires a reachable Postgres instance, e.g. `docker-services-compose.yml`'s
//! `postgres` service, with `DATABASE_URL` set to an admin-capable connection
//! string (`sqlx::test` creates and drops an ephemeral database per test):
//!
//! ```sh
//! DATABASE_URL=postgresql://postgres:postgres@127.0.0.1/haste_health cargo test -p haste-repository
//! ```

use sqlx::PgPool;

/// A sequence that has never had `nextval()` called on it has
/// `pg_sequence_last_value(...) = NULL`. Registering against it must not
/// error, and must report position 0 rather than propagating the NULL into
/// `pg_advisory_xact_lock_shared`, which would silently take no lock.
#[sqlx::test(migrations = "./pg-migrations")]
async fn register_on_unused_sequence_returns_zero_without_error(pool: PgPool) -> sqlx::Result<()> {
    let mut conn = pool.acquire().await?;

    sqlx::query("BEGIN").execute(&mut *conn).await?;
    let registered: i64 = sqlx::query_scalar("SELECT register_sequence_transaction($1)")
        .bind("resources_sequence_seq")
        .fetch_one(&mut *conn)
        .await?;
    sqlx::query("COMMIT").execute(&mut *conn).await?;

    assert_eq!(registered, 0);
    Ok(())
}

/// Basic contract sanity: after the sequence has advanced, registering
/// reports the current last_value, matching the pre-existing behavior.
#[sqlx::test(migrations = "./pg-migrations")]
async fn register_returns_current_last_value(pool: PgPool) -> sqlx::Result<()> {
    sqlx::query("CREATE SEQUENCE test_basic_seq")
        .execute(&pool)
        .await?;
    for _ in 0..3 {
        sqlx::query("SELECT nextval('test_basic_seq')")
            .execute(&pool)
            .await?;
    }

    let mut conn = pool.acquire().await?;
    sqlx::query("BEGIN").execute(&mut *conn).await?;
    let registered: i64 = sqlx::query_scalar("SELECT register_sequence_transaction($1)")
        .bind("test_basic_seq")
        .fetch_one(&mut *conn)
        .await?;
    sqlx::query("COMMIT").execute(&mut *conn).await?;

    assert_eq!(registered, 3);
    Ok(())
}

/// Both functions resolve the sequence name via a `regclass` cast, so a
/// nonexistent name must still fail loudly (just via Postgres's native
/// undefined_table error rather than the old hand-rolled RAISE).
#[sqlx::test(migrations = "./pg-migrations")]
async fn missing_sequence_raises_error(pool: PgPool) -> sqlx::Result<()> {
    let register_err = sqlx::query("SELECT register_sequence_transaction('does_not_exist_seq')")
        .execute(&pool)
        .await
        .unwrap_err();
    assert!(register_err.to_string().contains("does not exist"));

    let max_safe_err = sqlx::query("SELECT max_safe_seq('does_not_exist_seq')")
        .execute(&pool)
        .await
        .unwrap_err();
    assert!(max_safe_err.to_string().contains("does not exist"));

    Ok(())
}

/// The core end-to-end contract: a writer that has registered but not
/// committed must stay excluded from `max_safe_seq` even though the raw
/// sequence value has already moved past it, and the watermark must catch
/// up once that writer commits (releasing its advisory locks).
#[sqlx::test(migrations = "./pg-migrations")]
async fn max_safe_seq_excludes_uncommitted_writer_then_catches_up_after_commit(
    pool: PgPool,
) -> sqlx::Result<()> {
    sqlx::query("CREATE SEQUENCE test_visibility_seq")
        .execute(&pool)
        .await?;

    let mut writer = pool.acquire().await?;
    sqlx::query("BEGIN").execute(&mut *writer).await?;
    let registered: i64 = sqlx::query_scalar("SELECT register_sequence_transaction($1)")
        .bind("test_visibility_seq")
        .fetch_one(&mut *writer)
        .await?;
    assert_eq!(registered, 0);

    // Simulate the write that follows registration: the sequence advances,
    // but the transaction is still open.
    sqlx::query("SELECT nextval('test_visibility_seq')")
        .execute(&mut *writer)
        .await?;

    let safe_while_open: i64 = sqlx::query_scalar("SELECT max_safe_seq($1)")
        .bind("test_visibility_seq")
        .fetch_one(&pool)
        .await?;
    assert_eq!(
        safe_while_open, 0,
        "the uncommitted writer's row must not be considered safe yet"
    );

    sqlx::query("COMMIT").execute(&mut *writer).await?;

    let safe_after_commit: i64 = sqlx::query_scalar("SELECT max_safe_seq($1)")
        .bind("test_visibility_seq")
        .fetch_one(&pool)
        .await?;
    assert_eq!(
        safe_after_commit, 1,
        "once the writer commits and releases its locks, the watermark should catch up to last_value"
    );

    Ok(())
}

/// With multiple concurrent registrants, `max_safe_seq` must report the
/// lowest (oldest) still-open registration, not just any of them.
#[sqlx::test(migrations = "./pg-migrations")]
async fn max_safe_seq_reports_minimum_across_concurrent_registrants(
    pool: PgPool,
) -> sqlx::Result<()> {
    sqlx::query("CREATE SEQUENCE test_multi_seq")
        .execute(&pool)
        .await?;

    let mut writer_a = pool.acquire().await?;
    sqlx::query("BEGIN").execute(&mut *writer_a).await?;
    let a_registered: i64 = sqlx::query_scalar("SELECT register_sequence_transaction($1)")
        .bind("test_multi_seq")
        .fetch_one(&mut *writer_a)
        .await?;
    sqlx::query("SELECT nextval('test_multi_seq')")
        .execute(&mut *writer_a)
        .await?;
    assert_eq!(a_registered, 0);

    let mut writer_b = pool.acquire().await?;
    sqlx::query("BEGIN").execute(&mut *writer_b).await?;
    let b_registered: i64 = sqlx::query_scalar("SELECT register_sequence_transaction($1)")
        .bind("test_multi_seq")
        .fetch_one(&mut *writer_b)
        .await?;
    sqlx::query("SELECT nextval('test_multi_seq')")
        .execute(&mut *writer_b)
        .await?;
    assert_eq!(b_registered, 1);

    let safe_both_open: i64 = sqlx::query_scalar("SELECT max_safe_seq($1)")
        .bind("test_multi_seq")
        .fetch_one(&pool)
        .await?;
    assert_eq!(
        safe_both_open, 0,
        "must report the oldest open registration"
    );

    sqlx::query("COMMIT").execute(&mut *writer_a).await?;

    let safe_a_committed: i64 = sqlx::query_scalar("SELECT max_safe_seq($1)")
        .bind("test_multi_seq")
        .fetch_one(&pool)
        .await?;
    assert_eq!(
        safe_a_committed, 1,
        "with A committed, the watermark should advance to B's still-open registration"
    );

    sqlx::query("COMMIT").execute(&mut *writer_b).await?;

    let safe_all_committed: i64 = sqlx::query_scalar("SELECT max_safe_seq($1)")
        .bind("test_multi_seq")
        .fetch_one(&pool)
        .await?;
    assert_eq!(
        safe_all_committed, 2,
        "with nobody registered, falls back to the current sequence value"
    );

    Ok(())
}

/// `max_safe_seq` must only aggregate the *shared* bigint advisory locks
/// taken by `register_sequence_transaction`. An unrelated EXCLUSIVE bigint
/// advisory lock held by the same backend -- e.g. the rate limiter's
/// `pg_advisory_xact_lock(hashtext(key))` -- has the identical
/// (locktype='advisory', objsubid=1) shape and must be excluded via the
/// `mode = 'ShareLock'` filter, or it would corrupt the MIN computation.
#[sqlx::test(migrations = "./pg-migrations")]
async fn max_safe_seq_ignores_exclusive_advisory_locks_on_same_backend(
    pool: PgPool,
) -> sqlx::Result<()> {
    sqlx::query("CREATE SEQUENCE test_exclusive_seq")
        .execute(&pool)
        .await?;

    let mut writer = pool.acquire().await?;
    sqlx::query("BEGIN").execute(&mut *writer).await?;
    let registered: i64 = sqlx::query_scalar("SELECT register_sequence_transaction($1)")
        .bind("test_exclusive_seq")
        .fetch_one(&mut *writer)
        .await?;
    assert_eq!(registered, 0);

    // A deliberately tiny key: if the mode filter were missing, this would
    // win the MIN() and get reported instead of the real registration.
    sqlx::query("SELECT pg_advisory_xact_lock(-999999999999)")
        .execute(&mut *writer)
        .await?;

    let safe: i64 = sqlx::query_scalar("SELECT max_safe_seq($1)")
        .bind("test_exclusive_seq")
        .fetch_one(&pool)
        .await?;

    assert_eq!(
        safe, 0,
        "the unrelated exclusive lock must not affect the reported watermark"
    );

    Ok(())
}

/// Regression test for the fixed 2^32 overflow bug: the old query assumed
/// the bigint-form advisory lock always decomposes with `classid = 0`,
/// which only held while the registered value stayed under 2^32. The fixed
/// query reconstructs the full 64-bit key from `classid`/`objid`, so it
/// must correctly recover values past that boundary.
#[sqlx::test(migrations = "./pg-migrations")]
async fn max_safe_seq_reconstructs_values_past_2_32(pool: PgPool) -> sqlx::Result<()> {
    sqlx::query("CREATE SEQUENCE test_overflow_seq")
        .execute(&pool)
        .await?;

    const BIG_VAL: i64 = 5_000_000_000; // > 2^32 (4_294_967_296)

    let mut writer = pool.acquire().await?;
    sqlx::query("BEGIN").execute(&mut *writer).await?;
    // Mirror register_sequence_transaction's two locks directly, so we can
    // control the registered value without actually calling nextval() five
    // billion times.
    sqlx::query("SELECT pg_try_advisory_xact_lock_shared(('test_overflow_seq'::regclass)::int, 0)")
        .execute(&mut *writer)
        .await?;
    sqlx::query("SELECT pg_advisory_xact_lock_shared($1::bigint)")
        .bind(BIG_VAL)
        .execute(&mut *writer)
        .await?;

    let safe: i64 = sqlx::query_scalar("SELECT max_safe_seq($1)")
        .bind("test_overflow_seq")
        .fetch_one(&pool)
        .await?;

    assert_eq!(safe, BIG_VAL);
    Ok(())
}
