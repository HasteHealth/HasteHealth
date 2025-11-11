use crate::types::SupportedFHIRVersions;
use oxidized_jwt::{ProjectId, TenantId};

pub struct CreateProject {
    pub tenant: TenantId,
    pub id: Option<ProjectId>,
    pub fhir_version: SupportedFHIRVersions,
    pub system_created: bool,
}

#[derive(sqlx::FromRow, Debug)]
pub struct Project {
    pub tenant: TenantId,
    pub id: ProjectId,
    pub fhir_version: SupportedFHIRVersions,
    pub system_created: bool,
}

pub struct ProjectSearchClaims {
    pub id: Option<ProjectId>,
    pub fhir_version: Option<SupportedFHIRVersions>,
    pub system_created: Option<bool>,
}
