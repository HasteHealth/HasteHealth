use crate::{AuthorId, AuthorKind, ProjectId, TenantId, UserRole, VersionId, scopes::Scopes};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SubscriptionTier {
    #[serde(rename = "free")]
    Free,
    #[serde(rename = "professional")]
    Professional,
    #[serde(rename = "team")]
    Team,
    #[serde(rename = "unlimited")]
    Unlimited,
}

impl From<SubscriptionTier> for String {
    fn from(tier: SubscriptionTier) -> Self {
        match tier {
            SubscriptionTier::Free => "free".to_string(),
            SubscriptionTier::Professional => "professional".to_string(),
            SubscriptionTier::Team => "team".to_string(),
            SubscriptionTier::Unlimited => "unlimited".to_string(),
        }
    }
}

impl TryFrom<String> for SubscriptionTier {
    type Error = OperationOutcomeError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "free" => Ok(SubscriptionTier::Free),
            "professional" => Ok(SubscriptionTier::Professional),
            "team" => Ok(SubscriptionTier::Team),
            "unlimited" => Ok(SubscriptionTier::Unlimited),
            _ => Err(OperationOutcomeError::error(
                IssueType::Invalid(None),
                format!("Invalid subscription tier: '{}'", value),
            )),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserTokenClaims {
    pub sub: AuthorId,
    pub exp: usize,
    pub aud: String,
    pub scope: Scopes,

    #[serde(rename = "https://haste.health/tenant")]
    pub tenant: TenantId,
    #[serde(rename = "https://haste.health/subscription_tier")]
    pub subscription_tier: SubscriptionTier,
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
