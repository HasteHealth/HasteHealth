use std::path::PathBuf;

use haste_jwt::{ProjectId, TenantId};

pub fn tenant_route_string(tenant: &TenantId) -> PathBuf {
    ["/w", tenant.as_ref()].iter().collect()
}

pub fn project_route_string(tenant: &TenantId, project: &ProjectId) -> PathBuf {
    ["/w", tenant.as_ref(), project.as_ref(), "api", "v1"]
        .iter()
        .collect()
}

pub fn oidc_route_string(tenant: &TenantId, project: &ProjectId, path: &str) -> PathBuf {
    let route = project_route_string(tenant, project)
        .join("oidc")
        .join(path);
    route
}
