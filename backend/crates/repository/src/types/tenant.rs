use crate::types::TenantId;

pub struct CreateTenant {
    pub id: Option<TenantId>,
    pub subscription_tier: Option<String>,
}

#[derive(sqlx::FromRow, Debug)]
pub struct Tenant {
    pub id: String,
    pub subscription_tier: String,
}

pub struct TenantSearchClaims {
    pub subscription_tier: Option<String>,
}
