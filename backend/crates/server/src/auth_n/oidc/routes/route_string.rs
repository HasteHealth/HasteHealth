use oxidized_repository::types::{ProjectId, TenantId};

pub fn tenant_route_string(tenant: &TenantId) -> String {
    format!("/w/{}/api/v1", tenant)
}

pub fn project_route_string(tenant: &TenantId, project: &ProjectId) -> String {
    format!("/{}/{}", tenant_route_string(tenant), project.as_ref())
}

pub fn oidc_route_string(tenant: &TenantId, project: &ProjectId, path: &str) -> String {
    format!("/{}/oidc/{}", project_route_string(tenant, project), path)
}
