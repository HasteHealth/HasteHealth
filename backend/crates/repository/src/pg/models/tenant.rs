use crate::{
    admin::TenantModelAdmin,
    pg::{PGConnection, StoreError},
    types::tenant::{CreateTenant, Tenant, TenantSearchClaims},
    utilities::{generate_id, validate_id},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::TenantId;
use sqlx::{PgExecutor, QueryBuilder};

async fn create_tenant<'a, 'e, E>(
    executor: E,
    tenant: CreateTenant,
) -> Result<Tenant, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let id = tenant
        .id
        .unwrap_or_else(|| TenantId::new(generate_id(None)));
    validate_id(id.as_ref())?;

    let result = sqlx::query_as::<_, Tenant>(
        r"
            INSERT INTO tenants (id, subscription_tier)
            VALUES ($1, $2)
            RETURNING id, subscription_tier
        ",
    )
    .bind(id)
    .bind(
        tenant
            .subscription_tier
            .unwrap_or_else(|| "free".to_string()),
    )
    .fetch_one(executor)
    .await;

    match result {
        Ok(tenant) => Ok(tenant),
        Err(e) => {
            if let sqlx::Error::Database(db_error) = &e
                && db_error.code().as_deref() == Some("23505")
            {
                println!("Duplicate tenant ID detected");
                Err(StoreError::Duplicate.into())
            } else {
                Err(StoreError::SQLXError(e).into())
            }
        }
    }
}

async fn read_tenant<'a, 'e, E>(
    executor: E,
    id: &'a str,
) -> Result<Option<Tenant>, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let tenant = sqlx::query_as::<_, Tenant>(
        r"
            SELECT id, subscription_tier
            FROM tenants
            WHERE id = $1
        ",
    )
    .bind(id)
    .fetch_optional(executor)
    .await
    .map_err(StoreError::SQLXError)?;

    Ok(tenant)
}

async fn update_tenant<'a, 'e, E>(
    executor: E,
    tenant: Tenant,
) -> Result<Tenant, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let updated_tenant = sqlx::query_as::<_, Tenant>(
        r"
            UPDATE tenants
            SET subscription_tier = $1
            WHERE id = $2
            RETURNING id, subscription_tier
        ",
    )
    .bind(tenant.subscription_tier)
    .bind(tenant.id)
    .fetch_one(executor)
    .await
    .map_err(StoreError::SQLXError)?;

    Ok(updated_tenant)
}

async fn delete_tenant<'a, 'e, E>(executor: E, id: &'a str) -> Result<(), OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    sqlx::query(
        r"
            DELETE FROM tenants
            WHERE id = $1
        ",
    )
    .bind(id)
    .execute(executor)
    .await
    .map_err(StoreError::SQLXError)?;

    Ok(())
}

async fn search_tenant<'a, 'e, E>(
    executor: E,
    clauses: &'a TenantSearchClaims,
) -> Result<Vec<Tenant>, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let mut query_builder: QueryBuilder<sqlx::Postgres> =
        QueryBuilder::new(r"SELECT id, subscription_tier FROM tenants WHERE ");

    if let Some(subscription_tier) = clauses.subscription_tier.as_ref() {
        query_builder
            .push(" subscription_tier = ")
            .push_bind(subscription_tier);
    }

    let query = query_builder.build_query_as::<Tenant>();

    let tenants: Vec<Tenant> = query.fetch_all(executor).await.map_err(StoreError::from)?;

    Ok(tenants)
}

impl<Key: AsRef<str> + Send + Sync>
    TenantModelAdmin<CreateTenant, Tenant, TenantSearchClaims, Tenant, Key> for PGConnection
{
    async fn create(
        &self,
        _tenant: &TenantId,
        new_tenant: CreateTenant,
    ) -> Result<Tenant, OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => create_tenant(pool, new_tenant).await,
            PGConnection::Transaction(tx, _) => {
                let mut tx = tx.lock().await;
                create_tenant(&mut **tx, new_tenant).await
            }
        }
    }

    async fn read(
        &self,
        _tenant: &TenantId,
        id: &Key,
    ) -> Result<Option<Tenant>, haste_fhir_operation_error::OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => read_tenant(pool, id.as_ref()).await,
            PGConnection::Transaction(tx, _) => {
                let mut tx = tx.lock().await;
                read_tenant(&mut **tx, id.as_ref()).await
            }
        }
    }

    async fn update(
        &self,
        _tenant: &TenantId,
        model: Tenant,
    ) -> Result<Tenant, haste_fhir_operation_error::OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => update_tenant(pool, model).await,
            PGConnection::Transaction(tx, _) => {
                let mut tx = tx.lock().await;
                update_tenant(&mut **tx, model).await
            }
        }
    }

    async fn delete(
        &self,
        _tenant: &TenantId,
        id: &Key,
    ) -> Result<(), haste_fhir_operation_error::OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => delete_tenant(pool, id.as_ref()).await,
            PGConnection::Transaction(tx, _) => {
                let mut tx = tx.lock().await;
                delete_tenant(&mut **tx, id.as_ref()).await
            }
        }
    }

    async fn search(
        &self,
        _tenant: &TenantId,
        claims: &TenantSearchClaims,
    ) -> Result<Vec<Tenant>, OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => search_tenant(pool, claims).await,
            PGConnection::Transaction(tx, _) => {
                let mut tx = tx.lock().await;
                search_tenant(&mut **tx, claims).await
            }
        }
    }
}
