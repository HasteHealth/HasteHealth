use crate::types::{ProjectId, SupportedFHIRVersions, TenantId};

pub struct CreateProject {
    pub tenant: TenantId,
    pub id: Option<ProjectId>,
    pub fhir_version: SupportedFHIRVersions,
}

#[derive(sqlx::FromRow, Debug)]
pub struct Project {
    pub tenant: String,
    pub id: String,
    pub fhir_version: SupportedFHIRVersions,
}

pub struct ProjectSearchClaims {
    pub id: Option<ProjectId>,
    pub fhir_version: Option<SupportedFHIRVersions>,
}
