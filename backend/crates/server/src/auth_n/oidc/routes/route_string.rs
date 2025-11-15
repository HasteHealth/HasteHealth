use std::path::PathBuf;

use haste_jwt::{ProjectId, TenantId};

fn tenant_route_path(tenant: &TenantId) -> PathBuf {
    ["/w", tenant.as_ref(), "api", "v1"].iter().collect()
}

fn project_route_string(tenant: &TenantId, project: &ProjectId) -> PathBuf {
    tenant_route_path(tenant).join(project.as_ref())
}

pub fn oidc_route_string(tenant: &TenantId, project: &ProjectId, path: &str) -> PathBuf {
    let route = project_route_string(tenant, project)
        .join("oidc")
        .join(path);
    route
}
