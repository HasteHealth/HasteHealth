//! Integration tests for the `tenant_key` table and `TenantKeyProvider`
//! (see `pg-migrations/20261125120000_tenant_key.sql`).
//!
//! Requires a reachable Postgres instance, e.g. `docker-services-compose.yml`'s
//! `postgres` service, with `DATABASE_URL` set to an admin-capable connection
//! string (`sqlx::test` creates and drops an ephemeral database per test):
//!
//! ```sh
//! DATABASE_URL=postgresql://postgres:postgres@127.0.0.1/haste_health cargo test -p haste-repository
//! ```

use haste_jwt::TenantId;
use haste_repository::{
    fhir::FHIRRepository,
    pg::PGConnection,
    tenant_key::{TenantKey, TenantKeyProvider},
};
use sqlx::PgPool;

fn tenant(id: &str) -> TenantId {
    TenantId::new(id.to_string()).unwrap()
}

async fn create_tenant(pool: &PgPool, tenant: &str) -> sqlx::Result<()> {
    sqlx::query("INSERT INTO tenants (id) VALUES ($1)")
        .bind(tenant)
        .execute(pool)
        .await?;
    Ok(())
}

/// Retires the active key of `tenant` and makes `wrapped_key` the next
/// version, as a rotation does.
async fn rotate(pool: &PgPool, tenant: &str, wrapped_key: &[u8]) -> sqlx::Result<()> {
    let mut tx = pool.begin().await?;
    let retired: i32 = sqlx::query_scalar(
        r"
            UPDATE tenant_key SET state = 'retired', retired_at = now()
            WHERE tenant = $1 AND state = 'active'
            RETURNING version
        ",
    )
    .bind(tenant)
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO tenant_key (tenant, version, wrapped_key, wrapped_by) VALUES ($1, $2, $3, 'platform')",
    )
    .bind(tenant)
    .bind(retired + 1)
    .bind(wrapped_key)
    .execute(&mut *tx)
    .await?;
    tx.commit().await
}

async fn key_count(pool: &PgPool, tenant: &str) -> sqlx::Result<i64> {
    sqlx::query_scalar("SELECT count(*) FROM tenant_key WHERE tenant = $1")
        .bind(tenant)
        .fetch_one(pool)
        .await
}

#[sqlx::test(migrations = "./pg-migrations")]
async fn a_tenant_starts_without_a_key(pool: PgPool) -> sqlx::Result<()> {
    create_tenant(&pool, "acme").await?;
    let repo = PGConnection::pool(pool);

    assert_eq!(repo.active_tenant_key(&tenant("acme")).await.unwrap(), None);
    assert_eq!(repo.tenant_key(&tenant("acme"), 1).await.unwrap(), None);
    Ok(())
}

#[sqlx::test(migrations = "./pg-migrations")]
async fn the_first_key_is_version_one_and_active(pool: PgPool) -> sqlx::Result<()> {
    create_tenant(&pool, "acme").await?;
    let repo = PGConnection::pool(pool);

    let created = repo
        .create_first_tenant_key(&tenant("acme"), b"wrapped", "platform")
        .await
        .unwrap();

    let expected = TenantKey {
        version: 1,
        wrapped_key: b"wrapped".to_vec(),
        wrapped_by: "platform".to_string(),
    };
    assert_eq!(created, Some(expected.clone()));
    assert_eq!(
        repo.active_tenant_key(&tenant("acme")).await.unwrap(),
        Some(expected.clone())
    );
    assert_eq!(
        repo.tenant_key(&tenant("acme"), 1).await.unwrap(),
        Some(expected)
    );
    Ok(())
}

/// The second caller gets the key the first one stored, not its own.
#[sqlx::test(migrations = "./pg-migrations")]
async fn a_tenant_with_a_key_keeps_it(pool: PgPool) -> sqlx::Result<()> {
    create_tenant(&pool, "acme").await?;
    let repo = PGConnection::pool(pool.clone());

    repo.create_first_tenant_key(&tenant("acme"), b"first", "platform")
        .await
        .unwrap();
    let second = repo
        .create_first_tenant_key(&tenant("acme"), b"second", "platform")
        .await
        .unwrap()
        .unwrap();

    assert_eq!(second.version, 1);
    assert_eq!(second.wrapped_key, b"first");
    assert_eq!(key_count(&pool, "acme").await?, 1);
    Ok(())
}

/// Servers that need a tenant's first key at once all end up with one key.
#[sqlx::test(migrations = "./pg-migrations")]
async fn concurrent_first_keys_agree(pool: PgPool) -> sqlx::Result<()> {
    create_tenant(&pool, "acme").await?;
    let repo = PGConnection::pool(pool.clone());

    let creations = (0u8..8).map(|caller| {
        let repo = repo.clone();
        tokio::spawn(async move {
            repo.create_first_tenant_key(&tenant("acme"), &[caller; 4], "platform")
                .await
                .unwrap()
                .unwrap()
        })
    });
    let mut keys = Vec::new();
    for creation in creations.collect::<Vec<_>>() {
        keys.push(creation.await.unwrap());
    }

    assert!(keys.iter().all(|key| key == &keys[0]));
    assert_eq!(key_count(&pool, "acme").await?, 1);
    Ok(())
}

#[sqlx::test(migrations = "./pg-migrations")]
async fn each_tenant_has_its_own_key(pool: PgPool) -> sqlx::Result<()> {
    create_tenant(&pool, "acme").await?;
    create_tenant(&pool, "globex").await?;
    let repo = PGConnection::pool(pool);

    repo.create_first_tenant_key(&tenant("acme"), b"acme-key", "platform")
        .await
        .unwrap();
    assert_eq!(
        repo.active_tenant_key(&tenant("globex")).await.unwrap(),
        None
    );

    let globex = repo
        .create_first_tenant_key(&tenant("globex"), b"globex-key", "platform")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(globex.wrapped_key, b"globex-key");
    assert_eq!(
        repo.tenant_key(&tenant("acme"), 1)
            .await
            .unwrap()
            .unwrap()
            .wrapped_key,
        b"acme-key"
    );
    Ok(())
}

/// After a rotation the new version encrypts and the old one is still read.
#[sqlx::test(migrations = "./pg-migrations")]
async fn a_retired_key_is_read_by_version_and_is_not_active(pool: PgPool) -> sqlx::Result<()> {
    create_tenant(&pool, "acme").await?;
    let repo = PGConnection::pool(pool.clone());
    repo.create_first_tenant_key(&tenant("acme"), b"first", "platform")
        .await
        .unwrap();

    rotate(&pool, "acme", b"second").await?;

    let active = repo
        .active_tenant_key(&tenant("acme"))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        (active.version, active.wrapped_key.as_slice()),
        (2, &b"second"[..])
    );

    let retired = repo.tenant_key(&tenant("acme"), 1).await.unwrap().unwrap();
    assert_eq!(retired.wrapped_key, b"first");

    // A tenant that has keys is not given a "first" one again.
    let again = repo
        .create_first_tenant_key(&tenant("acme"), b"third", "platform")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(again.version, 2);
    assert_eq!(key_count(&pool, "acme").await?, 2);
    Ok(())
}

/// A key created inside a transaction would vanish on rollback while the
/// caller went on encrypting with it.
#[sqlx::test(migrations = "./pg-migrations")]
async fn a_transaction_does_not_create_a_key(pool: PgPool) -> sqlx::Result<()> {
    create_tenant(&pool, "acme").await?;
    let repo = PGConnection::pool(pool.clone());
    repo.create_first_tenant_key(&tenant("acme"), b"wrapped", "platform")
        .await
        .unwrap();
    create_tenant(&pool, "globex").await?;

    let transaction = repo.transaction(false).await.unwrap();
    assert!(
        transaction
            .create_first_tenant_key(&tenant("globex"), b"wrapped", "platform")
            .await
            .is_err()
    );
    // Reading is fine.
    assert_eq!(
        transaction
            .active_tenant_key(&tenant("acme"))
            .await
            .unwrap()
            .unwrap()
            .version,
        1
    );
    assert_eq!(
        transaction
            .tenant_key(&tenant("acme"), 1)
            .await
            .unwrap()
            .unwrap()
            .version,
        1
    );
    transaction.rollback().await.unwrap();

    assert_eq!(key_count(&pool, "globex").await?, 0);
    Ok(())
}

#[sqlx::test(migrations = "./pg-migrations")]
async fn a_tenant_has_one_active_key(pool: PgPool) -> sqlx::Result<()> {
    create_tenant(&pool, "acme").await?;
    let repo = PGConnection::pool(pool.clone());
    repo.create_first_tenant_key(&tenant("acme"), b"first", "platform")
        .await
        .unwrap();

    let second_active = sqlx::query(
        "INSERT INTO tenant_key (tenant, version, wrapped_key, wrapped_by) VALUES ('acme', 2, 'x', 'platform')",
    )
    .execute(&pool)
    .await
    .unwrap_err();

    assert!(second_active.to_string().contains("tenant_key_active_idx"));
    Ok(())
}

/// The retirement time is the record of a rotation: a retired key has one
/// and an active key does not.
#[sqlx::test(migrations = "./pg-migrations")]
async fn only_a_retired_key_has_a_retirement_time(pool: PgPool) -> sqlx::Result<()> {
    create_tenant(&pool, "acme").await?;
    let repo = PGConnection::pool(pool.clone());
    repo.create_first_tenant_key(&tenant("acme"), b"first", "platform")
        .await
        .unwrap();

    for update in [
        "UPDATE tenant_key SET state = 'retired' WHERE tenant = 'acme'",
        "UPDATE tenant_key SET retired_at = now() WHERE tenant = 'acme'",
        "UPDATE tenant_key SET state = 'lost' WHERE tenant = 'acme'",
    ] {
        assert!(
            sqlx::query(update).execute(&pool).await.is_err(),
            "{update}"
        );
    }
    Ok(())
}

#[sqlx::test(migrations = "./pg-migrations")]
async fn deleting_a_tenant_deletes_its_keys(pool: PgPool) -> sqlx::Result<()> {
    create_tenant(&pool, "acme").await?;
    create_tenant(&pool, "globex").await?;
    let repo = PGConnection::pool(pool.clone());
    for id in ["acme", "globex"] {
        repo.create_first_tenant_key(&tenant(id), b"wrapped", "platform")
            .await
            .unwrap();
    }

    sqlx::query("DELETE FROM tenants WHERE id = 'acme'")
        .execute(&pool)
        .await?;

    assert_eq!(key_count(&pool, "acme").await?, 0);
    assert_eq!(key_count(&pool, "globex").await?, 1);
    Ok(())
}
