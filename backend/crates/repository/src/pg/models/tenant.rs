use crate::{
    admin::TenantModelAdmin,
    pg::{PGConnection, StoreError},
    types::tenant::{CreateTenant, Tenant, TenantSearchClaims},
    utilities::{generate_id, validate_id},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::TenantId;
use sqlx::{PgExecutor, QueryBuilder};

fn validate_tenant_customization(
    subscription_tier: &str,
    display_name: Option<&String>,
    logo_data: Option<&Vec<u8>>,
    logo_content_type: Option<&String>,
) -> Result<(), OperationOutcomeError> {
    if logo_data.is_some() != logo_content_type.is_some() {
        return Err(OperationOutcomeError::error(
            haste_fhir_model::r4::generated::terminology::IssueType::invalid(),
            "Tenant logo data and content type must be provided together".to_string(),
        ));
    }

    if let Some(content_type) = logo_content_type
        && !content_type.starts_with("image/")
    {
        return Err(OperationOutcomeError::error(
            haste_fhir_model::r4::generated::terminology::IssueType::invalid(),
            "Tenant logo content type must be an image MIME type".to_string(),
        ));
    }

    if subscription_tier == "free"
        && (display_name.is_some() || logo_data.is_some() || logo_content_type.is_some())
    {
        return Err(OperationOutcomeError::error(
            haste_fhir_model::r4::generated::terminology::IssueType::forbidden(),
            "Tenant customization requires a paid subscription tier".to_string(),
        ));
    }

    Ok(())
}

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

    validate_tenant_customization(
        tenant.subscription_tier.as_deref().unwrap_or("free"),
        tenant.display_name.as_ref(),
        tenant.logo_data.as_ref(),
        tenant.logo_content_type.as_ref(),
    )?;

    let result = sqlx::query_as::<_, Tenant>(
        r"
            INSERT INTO tenants (id, subscription_tier, display_name, logo_data, logo_content_type)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, subscription_tier, display_name, logo_data, logo_content_type
        ",
    )
    .bind(id)
    .bind(
        tenant
            .subscription_tier
            .unwrap_or_else(|| "free".to_string()),
    )
    .bind(tenant.display_name)
    .bind(tenant.logo_data)
    .bind(tenant.logo_content_type)
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
            SELECT id, subscription_tier, display_name, logo_data, logo_content_type
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
    validate_tenant_customization(
        &tenant.subscription_tier,
        tenant.display_name.as_ref(),
        tenant.logo_data.as_ref(),
        tenant.logo_content_type.as_ref(),
    )?;

    let updated_tenant = sqlx::query_as::<_, Tenant>(
        r"
            UPDATE tenants
            SET subscription_tier = $1, display_name = $2, logo_data = $3, logo_content_type = $4
            WHERE id = $5
            RETURNING id, subscription_tier, display_name, logo_data, logo_content_type
        ",
    )
    .bind(tenant.subscription_tier)
    .bind(tenant.display_name)
    .bind(tenant.logo_data)
    .bind(tenant.logo_content_type)
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
    let mut query_builder: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
        r"SELECT id, subscription_tier, display_name, logo_data, logo_content_type FROM tenants WHERE ",
    );

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
            PGConnection::Transaction(tx, _, _) => {
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
            PGConnection::Transaction(tx, _, _) => {
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
            PGConnection::Transaction(tx, _, _) => {
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
            PGConnection::Transaction(tx, _, _) => {
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
            PGConnection::Transaction(tx, _, _) => {
                let mut tx = tx.lock().await;
                search_tenant(&mut **tx, claims).await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::validate_tenant_customization;

    #[test]
    fn free_tenants_cannot_set_branding() {
        let result =
            validate_tenant_customization("free", Some(&"Example Health".to_string()), None, None);

        assert!(result.is_err());
    }

    #[test]
    fn paid_tenants_can_set_branding() {
        let result = validate_tenant_customization(
            "professional",
            Some(&"Example Health".to_string()),
            Some(&vec![1, 2, 3]),
            Some(&"image/png".to_string()),
        );

        assert!(result.is_ok());
    }
}
