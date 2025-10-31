use crate::{AuthorId, AuthorKind, ProjectId, TenantId, UserRole, VersionId, scopes::Scopes};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserTokenClaims {
    pub sub: AuthorId,
    pub exp: usize,
    pub aud: String,
    pub scope: Scopes,

    #[serde(rename = "https://oxidized-health.app/tenant")]
    pub tenant: TenantId,
    #[serde(rename = "https://oxidized-health.app/project")]
    pub project: Option<ProjectId>,
    #[serde(rename = "https://oxidized-health.app/user_role")]
    pub user_role: UserRole,
    #[serde(rename = "https://oxidized-health.app/user_id")]
    pub user_id: AuthorId,
    #[serde(rename = "https://oxidized-health.app/resource_type")]
    pub resource_type: AuthorKind,
    #[serde(rename = "https://oxidized-health.app/access_policies")]
    pub access_policy_version_ids: Vec<VersionId>,
    #[serde(rename = "https://oxidized-health.app/membership_id")]
    pub membership_id: Option<String>,
}
