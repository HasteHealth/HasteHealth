use oxidized_fhir_client::request::Operation;
use oxidized_fhir_operation_error::OperationOutcomeError;
use sqlx::{Acquire, Postgres};

use crate::{
    TenantId,
    admin::TenantAuthAdmin,
    pg::{PGConnection, StoreError},
    utilities::generate_id,
};

struct CreateTenant {
    subscription_tier: Option<String>,
}

struct Tenant {
    id: String,
    subscription_tier: String,
}

struct TenantSearchClaims {
    subscription_tier: Option<String>,
}

fn create_tenant<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: CreateTenant,
) -> impl Future<Output = Result<Tenant, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;
        let tenant = sqlx::query_as!(
            Tenant,
            "INSERT INTO tenants (id, subscription_tier) VALUES ($1, $2) RETURNING id, subscription_tier",
            generate_id(),
            tenant.subscription_tier.unwrap_or("free".to_string())
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(StoreError::SQLXError)?;

        Ok(tenant)
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
        tenant: &crate::TenantId,
        id: &str,
    ) -> Result<Tenant, oxidized_fhir_operation_error::OperationOutcomeError> {
        todo!()
    }

    async fn update(
        &self,
        tenant: &crate::TenantId,
        model: Tenant,
    ) -> Result<Tenant, oxidized_fhir_operation_error::OperationOutcomeError> {
        todo!()
    }

    async fn delete(
        &self,
        tenant: &crate::TenantId,
        id: &str,
    ) -> Result<Tenant, oxidized_fhir_operation_error::OperationOutcomeError> {
        todo!()
    }

    async fn search(
        &self,
        tenant: &TenantId,
        claims: &TenantSearchClaims,
    ) -> Result<Vec<Tenant>, OperationOutcomeError> {
        todo!();
    }
}
