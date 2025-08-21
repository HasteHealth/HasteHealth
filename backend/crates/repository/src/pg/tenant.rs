use crate::{
    admin::TenantAuthAdmin,
    pg::{PGConnection, StoreError},
    types::{
        TenantId,
        tenant::{CreateTenant, Tenant, TenantSearchClaims},
    },
    utilities::generate_id,
};
use oxidized_fhir_operation_error::OperationOutcomeError;
use sqlx::{Acquire, Postgres, QueryBuilder};

fn create_tenant<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: CreateTenant,
) -> impl Future<Output = Result<Tenant, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;
        let id = tenant.id.unwrap_or(TenantId::new(generate_id(None)));
        let tenant = sqlx::query_as!(
            Tenant,
            "INSERT INTO tenants (id, subscription_tier) VALUES ($1, $2) RETURNING id, subscription_tier",
            id.as_ref(),
            tenant.subscription_tier.unwrap_or("free".to_string())
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(StoreError::SQLXError)?;

        Ok(tenant)
    }
}

fn read_tenant<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    id: &'a str,
) -> impl Future<Output = Result<Tenant, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;
        let tenant = sqlx::query_as!(
            Tenant,
            r#"SELECT id, subscription_tier FROM tenants where id = $1"#,
            id
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(StoreError::SQLXError)?;

        Ok(tenant)
    }
}

fn update_tenant<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: Tenant,
) -> impl Future<Output = Result<Tenant, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;
        let updated_tenant = sqlx::query_as!(
            Tenant,
            "UPDATE tenants SET subscription_tier = $1 WHERE id = $2 RETURNING id, subscription_tier",
            tenant.subscription_tier,
            tenant.id
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(StoreError::SQLXError)?;

        Ok(updated_tenant)
    }
}

fn delete_tenant<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    id: &'a str,
) -> impl Future<Output = Result<Tenant, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;
        let deleted_tenant = sqlx::query_as!(
            Tenant,
            "DELETE FROM tenants WHERE id = $1 RETURNING id, subscription_tier",
            id
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(StoreError::SQLXError)?;

        Ok(deleted_tenant)
    }
}

fn search_tenant<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    clauses: &'a TenantSearchClaims,
) -> impl Future<Output = Result<Vec<Tenant>, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;
        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new(r#"SELECT id, subscription_tier FROM tenants WHERE "#);

        if let Some(subscription_tier) = clauses.subscription_tier.as_ref() {
            query_builder
                .push(" subscription_tier = ")
                .push_bind(subscription_tier);
        }

        let query = query_builder.build_query_as();

        let tenants: Vec<Tenant> = query
            .fetch_all(&mut *conn)
            .await
            .map_err(StoreError::from)?;

        Ok(tenants)
    }
}

impl TenantAuthAdmin<CreateTenant, Tenant, TenantSearchClaims> for PGConnection {
    async fn create(
        &self,
        _tenant: &TenantId,
        new_tenant: CreateTenant,
    ) -> Result<Tenant, OperationOutcomeError> {
        match self {
            PGConnection::PgPool(pool) => {
                let res = create_tenant(pool, new_tenant).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut tx = tx.lock().await;
                let res = create_tenant(&mut *tx, new_tenant).await?;
                Ok(res)
            }
        }
    }

    async fn read(
        &self,
        _tenant: &TenantId,
        id: &str,
    ) -> Result<Tenant, oxidized_fhir_operation_error::OperationOutcomeError> {
        match self {
            PGConnection::PgPool(pool) => {
                let res = read_tenant(pool, id).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut tx = tx.lock().await;
                let res = read_tenant(&mut *tx, id).await?;
                Ok(res)
            }
        }
    }

    async fn update(
        &self,
        _tenant: &TenantId,
        model: Tenant,
    ) -> Result<Tenant, oxidized_fhir_operation_error::OperationOutcomeError> {
        match self {
            PGConnection::PgPool(pool) => {
                let res = update_tenant(pool, model).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut tx = tx.lock().await;
                let res = update_tenant(&mut *tx, model).await?;
                Ok(res)
            }
        }
    }

    async fn delete(
        &self,
        _tenant: &TenantId,
        id: &str,
    ) -> Result<Tenant, oxidized_fhir_operation_error::OperationOutcomeError> {
        match self {
            PGConnection::PgPool(pool) => {
                let res = delete_tenant(pool, id).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut tx = tx.lock().await;
                let res = delete_tenant(&mut *tx, id).await?;
                Ok(res)
            }
        }
    }

    async fn search(
        &self,
        _tenant: &TenantId,
        claims: &TenantSearchClaims,
    ) -> Result<Vec<Tenant>, OperationOutcomeError> {
        match self {
            PGConnection::PgPool(pool) => {
                let res = search_tenant(pool, claims).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut tx = tx.lock().await;
                let res = search_tenant(&mut *tx, claims).await?;
                Ok(res)
            }
        }
    }
}
