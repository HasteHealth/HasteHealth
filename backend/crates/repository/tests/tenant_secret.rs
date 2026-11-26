//! Integration tests for the `tenant_secret` table and `TenantSecretProvider`
//! (see `pg-migrations/20261126120000_tenant_secret.sql`).
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
    tenant_key::TenantKeyProvider,
    tenant_secret::{TenantSecret, TenantSecretKind, TenantSecretProvider},
};
use sqlx::PgPool;

use TenantSecretKind::{IdpClientSecret, MfaTotp};

fn tenant(id: &str) -> TenantId {
    TenantId::new(id.to_string()).unwrap()
}

/// What an encryption under key version `key_version` would have produced.
fn encrypted(key_version: i32, ciphertext: &[u8]) -> TenantSecret {
    TenantSecret {
        key_version,
        nonce: b"nonce".to_vec(),
        ciphertext: ciphertext.to_vec(),
    }
}

/// A tenant with its first key, version 1.
async fn create_tenant(pool: &PgPool, id: &str) -> sqlx::Result<()> {
    sqlx::query("INSERT INTO tenants (id) VALUES ($1)")
        .bind(id)
        .execute(pool)
        .await?;
    PGConnection::pool(pool.clone())
        .create_first_tenant_key(&tenant(id), b"wrapped", "platform")
        .await
        .unwrap();
    Ok(())
}

/// Retires the active key of `tenant` and makes the next version the active
/// one, as a rotation does.
async fn rotate(pool: &PgPool, tenant: &str) -> sqlx::Result<()> {
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
        "INSERT INTO tenant_key (tenant, version, wrapped_key, wrapped_by) VALUES ($1, $2, 'wrapped', 'platform')",
    )
    .bind(tenant)
    .bind(retired + 1)
    .execute(&mut *tx)
    .await?;
    tx.commit().await
}

async fn secret_count(pool: &PgPool, tenant: &str) -> sqlx::Result<i64> {
    sqlx::query_scalar("SELECT count(*) FROM tenant_secret WHERE tenant = $1")
        .bind(tenant)
        .fetch_one(pool)
        .await
}

#[sqlx::test(migrations = "./pg-migrations")]
async fn an_owner_starts_without_a_secret(pool: PgPool) -> sqlx::Result<()> {
    create_tenant(&pool, "acme").await?;
    let repo = PGConnection::pool(pool);

    assert_eq!(
        repo.tenant_secret(&tenant("acme"), MfaTotp, "credential-1")
            .await
            .unwrap(),
        None
    );
    assert!(
        !repo
            .delete_tenant_secret(&tenant("acme"), MfaTotp, "credential-1")
            .await
            .unwrap()
    );
    Ok(())
}

#[sqlx::test(migrations = "./pg-migrations")]
async fn a_stored_secret_is_read_back(pool: PgPool) -> sqlx::Result<()> {
    create_tenant(&pool, "acme").await?;
    let repo = PGConnection::pool(pool);
    let secret = encrypted(1, b"ciphertext");

    repo.put_tenant_secret(&tenant("acme"), MfaTotp, "credential-1", &secret)
        .await
        .unwrap();

    assert_eq!(
        repo.tenant_secret(&tenant("acme"), MfaTotp, "credential-1")
            .await
            .unwrap(),
        Some(secret)
    );
    Ok(())
}

/// Every kind the code knows is one the table accepts.
#[sqlx::test(migrations = "./pg-migrations")]
async fn every_kind_is_stored(pool: PgPool) -> sqlx::Result<()> {
    create_tenant(&pool, "acme").await?;
    let repo = PGConnection::pool(pool.clone());

    for kind in [MfaTotp, IdpClientSecret] {
        repo.put_tenant_secret(&tenant("acme"), kind, "owner", &encrypted(1, b"ciphertext"))
            .await
            .unwrap();
    }

    let kinds: Vec<String> =
        sqlx::query_scalar("SELECT kind FROM tenant_secret WHERE tenant = 'acme' ORDER BY kind")
            .fetch_all(&pool)
            .await?;
    assert_eq!(kinds, [IdpClientSecret.as_str(), MfaTotp.as_str()]);
    Ok(())
}

/// A second put replaces the secret, under the key version it came with, and
/// records when.
#[sqlx::test(migrations = "./pg-migrations")]
async fn a_put_replaces_the_secret(pool: PgPool) -> sqlx::Result<()> {
    create_tenant(&pool, "acme").await?;
    let repo = PGConnection::pool(pool.clone());
    repo.put_tenant_secret(
        &tenant("acme"),
        MfaTotp,
        "credential-1",
        &encrypted(1, b"first"),
    )
    .await
    .unwrap();
    sqlx::query("UPDATE tenant_secret SET set_at = now() - interval '1 day'")
        .execute(&pool)
        .await?;
    rotate(&pool, "acme").await?;

    let second = TenantSecret {
        key_version: 2,
        nonce: b"other-nonce".to_vec(),
        ciphertext: b"second".to_vec(),
    };
    repo.put_tenant_secret(&tenant("acme"), MfaTotp, "credential-1", &second)
        .await
        .unwrap();

    assert_eq!(
        repo.tenant_secret(&tenant("acme"), MfaTotp, "credential-1")
            .await
            .unwrap(),
        Some(second)
    );
    assert_eq!(secret_count(&pool, "acme").await?, 1);
    let set_recently: bool =
        sqlx::query_scalar("SELECT set_at > now() - interval '1 hour' FROM tenant_secret")
            .fetch_one(&pool)
            .await?;
    assert!(set_recently);
    Ok(())
}

/// The tenant, the kind and the owner each tell secrets apart.
#[sqlx::test(migrations = "./pg-migrations")]
async fn secrets_are_kept_apart_by_tenant_kind_and_owner(pool: PgPool) -> sqlx::Result<()> {
    create_tenant(&pool, "acme").await?;
    create_tenant(&pool, "globex").await?;
    let repo = PGConnection::pool(pool);
    let secrets = [
        ("acme", MfaTotp, "one", &b"acme-totp-one"[..]),
        ("acme", MfaTotp, "two", b"acme-totp-two"),
        ("acme", IdpClientSecret, "one", b"acme-idp-one"),
        ("globex", MfaTotp, "one", b"globex-totp-one"),
    ];
    for (tenant_id, kind, owner, ciphertext) in secrets {
        repo.put_tenant_secret(&tenant(tenant_id), kind, owner, &encrypted(1, ciphertext))
            .await
            .unwrap();
    }

    for (tenant_id, kind, owner, ciphertext) in secrets {
        let stored = repo
            .tenant_secret(&tenant(tenant_id), kind, owner)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(stored.ciphertext, ciphertext);
    }

    // Deleting one leaves the others.
    assert!(
        repo.delete_tenant_secret(&tenant("acme"), MfaTotp, "one")
            .await
            .unwrap()
    );
    for (tenant_id, kind, owner, _) in secrets {
        let stored = repo
            .tenant_secret(&tenant(tenant_id), kind, owner)
            .await
            .unwrap();
        assert_eq!(
            stored.is_some(),
            (tenant_id, kind, owner) != ("acme", MfaTotp, "one"),
            "{tenant_id} {kind:?} {owner}"
        );
    }
    // And it is gone: there is nothing left to delete.
    assert!(
        !repo
            .delete_tenant_secret(&tenant("acme"), MfaTotp, "one")
            .await
            .unwrap()
    );
    Ok(())
}

/// A secret is stored under a key version its own tenant has.
#[sqlx::test(migrations = "./pg-migrations")]
async fn a_secret_needs_a_key_version_of_its_tenant(pool: PgPool) -> sqlx::Result<()> {
    create_tenant(&pool, "acme").await?;
    create_tenant(&pool, "globex").await?;
    rotate(&pool, "globex").await?;
    let repo = PGConnection::pool(pool.clone());

    // Version 2 exists for globex only.
    assert!(
        repo.put_tenant_secret(
            &tenant("acme"),
            MfaTotp,
            "credential-1",
            &encrypted(2, b"x")
        )
        .await
        .is_err()
    );
    assert_eq!(secret_count(&pool, "acme").await?, 0);

    repo.put_tenant_secret(
        &tenant("globex"),
        MfaTotp,
        "credential-1",
        &encrypted(2, b"x"),
    )
    .await
    .unwrap();
    Ok(())
}

/// A key version stays for as long as a secret is encrypted under it.
#[sqlx::test(migrations = "./pg-migrations")]
async fn a_key_version_in_use_is_not_removed(pool: PgPool) -> sqlx::Result<()> {
    create_tenant(&pool, "acme").await?;
    let repo = PGConnection::pool(pool.clone());
    repo.put_tenant_secret(
        &tenant("acme"),
        MfaTotp,
        "credential-1",
        &encrypted(1, b"x"),
    )
    .await
    .unwrap();
    rotate(&pool, "acme").await?;

    let delete_retired_key = "DELETE FROM tenant_key WHERE tenant = 'acme' AND version = 1";
    let refused = sqlx::query(delete_retired_key)
        .execute(&pool)
        .await
        .unwrap_err();
    assert!(refused.to_string().contains("fk_tenant_key"), "{refused}");

    // Once the secret is under the new version, the old one can go.
    repo.put_tenant_secret(
        &tenant("acme"),
        MfaTotp,
        "credential-1",
        &encrypted(2, b"y"),
    )
    .await
    .unwrap();
    let removed = sqlx::query(delete_retired_key).execute(&pool).await?;
    assert_eq!(removed.rows_affected(), 1);
    Ok(())
}

#[sqlx::test(migrations = "./pg-migrations")]
async fn deleting_a_tenant_deletes_its_secrets(pool: PgPool) -> sqlx::Result<()> {
    create_tenant(&pool, "acme").await?;
    create_tenant(&pool, "globex").await?;
    let repo = PGConnection::pool(pool.clone());
    for id in ["acme", "globex"] {
        for kind in [MfaTotp, IdpClientSecret] {
            repo.put_tenant_secret(&tenant(id), kind, "owner", &encrypted(1, b"x"))
                .await
                .unwrap();
        }
    }

    sqlx::query("DELETE FROM tenants WHERE id = 'acme'")
        .execute(&pool)
        .await?;

    assert_eq!(secret_count(&pool, "acme").await?, 0);
    assert_eq!(secret_count(&pool, "globex").await?, 2);
    Ok(())
}

#[sqlx::test(migrations = "./pg-migrations")]
async fn an_unknown_kind_is_refused(pool: PgPool) -> sqlx::Result<()> {
    create_tenant(&pool, "acme").await?;

    let refused = sqlx::query(
        "INSERT INTO tenant_secret (tenant, kind, owner, ciphertext, nonce, key_version) VALUES ('acme', 'password', 'owner', 'x', 'x', 1)",
    )
    .execute(&pool)
    .await
    .unwrap_err();

    assert!(
        refused.to_string().contains("tenant_secret_kind"),
        "{refused}"
    );
    Ok(())
}

/// Inside a transaction a secret is stored, replaced and removed with the
/// rest of the transaction.
#[sqlx::test(migrations = "./pg-migrations")]
async fn a_transaction_commits_or_rolls_back_its_secrets(pool: PgPool) -> sqlx::Result<()> {
    create_tenant(&pool, "acme").await?;
    let repo = PGConnection::pool(pool.clone());
    let acme = tenant("acme");
    repo.put_tenant_secret(&acme, MfaTotp, "kept", &encrypted(1, b"before"))
        .await
        .unwrap();

    let transaction = repo.transaction(false).await.unwrap();
    transaction
        .put_tenant_secret(&acme, MfaTotp, "new", &encrypted(1, b"new"))
        .await
        .unwrap();
    assert!(
        transaction
            .delete_tenant_secret(&acme, MfaTotp, "kept")
            .await
            .unwrap()
    );
    // The transaction reads its own writes, and nobody else does yet.
    assert!(
        transaction
            .tenant_secret(&acme, MfaTotp, "new")
            .await
            .unwrap()
            .is_some()
    );
    assert_eq!(
        repo.tenant_secret(&acme, MfaTotp, "new").await.unwrap(),
        None
    );
    transaction.rollback().await.unwrap();

    assert_eq!(
        repo.tenant_secret(&acme, MfaTotp, "new").await.unwrap(),
        None
    );
    assert_eq!(
        repo.tenant_secret(&acme, MfaTotp, "kept").await.unwrap(),
        Some(encrypted(1, b"before"))
    );

    let transaction = repo.transaction(false).await.unwrap();
    transaction
        .put_tenant_secret(&acme, MfaTotp, "new", &encrypted(1, b"new"))
        .await
        .unwrap();
    transaction.commit().await.unwrap();

    assert_eq!(
        repo.tenant_secret(&acme, MfaTotp, "new").await.unwrap(),
        Some(encrypted(1, b"new"))
    );
    Ok(())
}

/// A tenant's first key is created outside of the transaction that needs it
/// (see `create_first_tenant_key`), so after the transaction began. A secret
/// stored in that transaction can still be under that key.
#[sqlx::test(migrations = "./pg-migrations")]
async fn a_transaction_stores_under_a_key_created_after_it_began(pool: PgPool) -> sqlx::Result<()> {
    sqlx::query("INSERT INTO tenants (id) VALUES ('acme')")
        .execute(&pool)
        .await?;
    let repo = PGConnection::pool(pool);
    let acme = tenant("acme");

    let transaction = repo.transaction(false).await.unwrap();
    assert_eq!(transaction.active_tenant_key(&acme).await.unwrap(), None);
    repo.create_first_tenant_key(&acme, b"wrapped", "platform")
        .await
        .unwrap();
    transaction
        .put_tenant_secret(&acme, MfaTotp, "credential-1", &encrypted(1, b"x"))
        .await
        .unwrap();
    transaction.commit().await.unwrap();

    assert_eq!(
        repo.tenant_secret(&acme, MfaTotp, "credential-1")
            .await
            .unwrap(),
        Some(encrypted(1, b"x"))
    );
    Ok(())
}
