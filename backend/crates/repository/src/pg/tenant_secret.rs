use crate::{
    pg::{PGConnection, StoreError},
    tenant_secret::{TenantSecret, TenantSecretKind, TenantSecretProvider},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::TenantId;
use sqlx::PgExecutor;

/// Inserts the secret, or replaces the one the owner has. A replacement is a
/// new secret, so it is given a new `set_at`.
async fn put<'e, E>(
    executor: E,
    tenant: &TenantId,
    kind: TenantSecretKind,
    owner: &str,
    secret: &TenantSecret,
) -> Result<(), OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    sqlx::query(
        r"
            INSERT INTO tenant_secret (tenant, kind, owner, ciphertext, nonce, key_version)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (tenant, kind, owner) DO UPDATE SET
                ciphertext = EXCLUDED.ciphertext,
                nonce = EXCLUDED.nonce,
                key_version = EXCLUDED.key_version,
                set_at = now()
        ",
    )
    .bind(tenant.as_ref())
    .bind(kind.as_str())
    .bind(owner)
    .bind(&secret.ciphertext)
    .bind(&secret.nonce)
    .bind(secret.key_version)
    .execute(executor)
    .await
    .map_err(StoreError::from)?;

    Ok(())
}

async fn get<'e, E>(
    executor: E,
    tenant: &TenantId,
    kind: TenantSecretKind,
    owner: &str,
) -> Result<Option<TenantSecret>, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let secret = sqlx::query_as::<_, TenantSecret>(
        r"
            SELECT key_version, nonce, ciphertext
            FROM tenant_secret
            WHERE tenant = $1 AND kind = $2 AND owner = $3
        ",
    )
    .bind(tenant.as_ref())
    .bind(kind.as_str())
    .bind(owner)
    .fetch_optional(executor)
    .await
    .map_err(StoreError::from)?;

    Ok(secret)
}

async fn delete<'e, E>(
    executor: E,
    tenant: &TenantId,
    kind: TenantSecretKind,
    owner: &str,
) -> Result<bool, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let result =
        sqlx::query("DELETE FROM tenant_secret WHERE tenant = $1 AND kind = $2 AND owner = $3")
            .bind(tenant.as_ref())
            .bind(kind.as_str())
            .bind(owner)
            .execute(executor)
            .await
            .map_err(StoreError::from)?;

    Ok(result.rows_affected() == 1)
}

impl TenantSecretProvider for PGConnection {
    async fn put_tenant_secret(
        &self,
        tenant: &TenantId,
        kind: TenantSecretKind,
        owner: &str,
        secret: &TenantSecret,
    ) -> Result<(), OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => put(pool, tenant, kind, owner, secret).await,
            PGConnection::Transaction(tx, _, _) => {
                let mut tx = tx.lock().await;
                put(&mut **tx, tenant, kind, owner, secret).await
            }
        }
    }

    async fn tenant_secret(
        &self,
        tenant: &TenantId,
        kind: TenantSecretKind,
        owner: &str,
    ) -> Result<Option<TenantSecret>, OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => get(pool, tenant, kind, owner).await,
            PGConnection::Transaction(tx, _, _) => {
                let mut tx = tx.lock().await;
                get(&mut **tx, tenant, kind, owner).await
            }
        }
    }

    async fn delete_tenant_secret(
        &self,
        tenant: &TenantId,
        kind: TenantSecretKind,
        owner: &str,
    ) -> Result<bool, OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => delete(pool, tenant, kind, owner).await,
            PGConnection::Transaction(tx, _, _) => {
                let mut tx = tx.lock().await;
                delete(&mut **tx, tenant, kind, owner).await
            }
        }
    }
}
