use crate::{
    pg::{PGConnection, StoreError},
    tenant_key::{TenantKey, TenantKeyProvider},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::TenantId;
use sqlx::PgExecutor;

async fn active<'e, E>(
    executor: E,
    tenant: &TenantId,
) -> Result<Option<TenantKey>, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let key = sqlx::query_as::<_, TenantKey>(
        r"
            SELECT version, wrapped_key, wrapped_by
            FROM tenant_key
            WHERE tenant = $1 AND state = 'active'
        ",
    )
    .bind(tenant.as_ref())
    .fetch_optional(executor)
    .await
    .map_err(StoreError::from)?;

    Ok(key)
}

async fn at_version<'e, E>(
    executor: E,
    tenant: &TenantId,
    version: i32,
) -> Result<Option<TenantKey>, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let key = sqlx::query_as::<_, TenantKey>(
        r"
            SELECT version, wrapped_key, wrapped_by
            FROM tenant_key
            WHERE tenant = $1 AND version = $2
        ",
    )
    .bind(tenant.as_ref())
    .bind(version)
    .fetch_optional(executor)
    .await
    .map_err(StoreError::from)?;

    Ok(key)
}

/// Inserts version 1; the primary key refuses it when the tenant has a key
/// already. A concurrent insert of the same row is waited for, so whichever
/// caller is refused finds the other's key committed when it reads next.
async fn insert_first<'e, E>(
    executor: E,
    tenant: &TenantId,
    wrapped_key: &[u8],
    wrapped_by: &str,
) -> Result<(), OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    sqlx::query(
        r"
            INSERT INTO tenant_key (tenant, version, wrapped_key, wrapped_by)
            VALUES ($1, 1, $2, $3)
            ON CONFLICT DO NOTHING
        ",
    )
    .bind(tenant.as_ref())
    .bind(wrapped_key)
    .bind(wrapped_by)
    .execute(executor)
    .await
    .map_err(StoreError::from)?;

    Ok(())
}

impl TenantKeyProvider for PGConnection {
    async fn active_tenant_key(
        &self,
        tenant: &TenantId,
    ) -> Result<Option<TenantKey>, OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => active(pool, tenant).await,
            PGConnection::Transaction(tx, _, _) => {
                let mut tx = tx.lock().await;
                active(&mut **tx, tenant).await
            }
        }
    }

    async fn tenant_key(
        &self,
        tenant: &TenantId,
        version: i32,
    ) -> Result<Option<TenantKey>, OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => at_version(pool, tenant, version).await,
            PGConnection::Transaction(tx, _, _) => {
                let mut tx = tx.lock().await;
                at_version(&mut **tx, tenant, version).await
            }
        }
    }

    async fn create_first_tenant_key(
        &self,
        tenant: &TenantId,
        wrapped_key: &[u8],
        wrapped_by: &str,
    ) -> Result<Option<TenantKey>, OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => {
                insert_first(pool, tenant, wrapped_key, wrapped_by).await?;
                active(pool, tenant).await
            }
            // The caller keeps the key it gets and encrypts with it. Were the
            // transaction to roll back, the key would be gone and what it
            // encrypted lost with it.
            PGConnection::Transaction(..) => Err(StoreError::TenantKeyInTransaction.into()),
        }
    }
}
