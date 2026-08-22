use haste_jwt::TenantId;

pub struct CreateTenant {
    pub id: Option<TenantId>,
    pub subscription_tier: Option<String>,
    pub display_name: Option<String>,
    pub logo_data: Option<Vec<u8>>,
    pub logo_content_type: Option<String>,
}

#[derive(sqlx::FromRow, Debug)]
pub struct Tenant {
    pub id: TenantId,
    pub subscription_tier: String,
    pub display_name: Option<String>,
    pub logo_data: Option<Vec<u8>>,
    pub logo_content_type: Option<String>,
}

pub struct TenantSearchClaims {
    pub subscription_tier: Option<String>,
}
