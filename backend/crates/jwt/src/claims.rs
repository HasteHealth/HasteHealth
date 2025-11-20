use crate::{AuthorId, AuthorKind, ProjectId, TenantId, UserRole, VersionId, scopes::Scopes};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserTokenClaims {
    pub sub: AuthorId,
    pub exp: usize,
    pub aud: String,
    pub scope: Scopes,

    #[serde(rename = "https://haste.health/tenant")]
    pub tenant: TenantId,
    #[serde(rename = "https://haste.health/project")]
    pub project: Option<ProjectId>,
    #[serde(rename = "https://haste.health/user_role")]
    pub user_role: UserRole,
    #[serde(rename = "https://haste.health/user_id")]
    pub user_id: AuthorId,
    #[serde(rename = "https://haste.health/resource_type")]
    pub resource_type: AuthorKind,
    #[serde(rename = "https://haste.health/access_policies")]
    pub access_policy_version_ids: Vec<VersionId>,
    #[serde(rename = "https://haste.health/membership")]
    pub membership: Option<String>,
}
