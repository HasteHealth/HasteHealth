use oxidized_jwt::TenantId;

pub struct CreateTenant {
    pub id: Option<TenantId>,
    pub subscription_tier: Option<String>,
}

#[derive(sqlx::FromRow, Debug)]
pub struct Tenant {
    pub id: TenantId,
    pub subscription_tier: String,
}

pub struct TenantSearchClaims {
    pub subscription_tier: Option<String>,
}
