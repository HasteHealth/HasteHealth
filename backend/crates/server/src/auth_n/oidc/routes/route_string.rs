use std::path::PathBuf;

use haste_jwt::{ProjectId, TenantId};

fn project_route_string(tenant: &TenantId, project: &ProjectId) -> PathBuf {
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
