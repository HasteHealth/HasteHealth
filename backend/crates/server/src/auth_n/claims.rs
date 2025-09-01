use oxidized_repository::types::{ProjectId, ResourceId, TenantId, VersionId, user::UserRole};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum UserResourceTypes {
    Membership,
    ClientApplication,
    OperationDefinition,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserTokenClaims {
    pub sub: ResourceId,
    pub exp: usize,
    pub aud: String,
    pub scope: String,

    #[serde(rename = "https://oxidized-health.app/tenant")]
    pub tenant: TenantId,
    #[serde(rename = "https://oxidized-health.app/project")]
    pub project: Option<ProjectId>,
    #[serde(rename = "https://oxidized-health.app/user_role")]
    pub user_role: UserRole,
    #[serde(rename = "https://oxidized-health.app/user_id")]
    pub user_id: Option<ResourceId>,
    #[serde(rename = "https://oxidized-health.app/resource_type")]
    pub resource_type: UserResourceTypes,
    #[serde(rename = "https://oxidized-health.app/access_policies")]
    pub access_policy_version_ids: Vec<VersionId>,
}
