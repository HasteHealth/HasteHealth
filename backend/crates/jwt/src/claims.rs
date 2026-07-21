use crate::{AuthorId, AuthorKind, ProjectId, TenantId, UserRole, VersionId, scopes::Scopes};
use derivative::Derivative;
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
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
            SubscriptionTier::Free => "free",
            SubscriptionTier::Professional => "professional",
            SubscriptionTier::Team => "team",
            SubscriptionTier::Unlimited => "unlimited",
        }
        .to_string()
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
                IssueType::INVALID,
                format!("Invalid subscription tier: '{value}'"),
            )),
        }
    }
}

#[derive(Derivative, Serialize, Deserialize, Clone)]
#[derivative(Debug = "transparent")]
pub struct UserTokenClaims {
    pub sub: AuthorId,
    #[derivative(Debug = "ignore")]
    pub exp: usize,
    #[derivative(Debug = "ignore")]
    pub aud: String,
    #[derivative(Debug = "ignore")]
    pub scope: Scopes,

    #[serde(rename = "https://haste.health/tenant")]
    #[derivative(Debug = "ignore")]
    pub tenant: TenantId,
    #[serde(rename = "https://haste.health/subscription_tier")]
    #[derivative(Debug = "ignore")]
    pub subscription_tier: SubscriptionTier,
    #[serde(rename = "https://haste.health/project")]
    #[derivative(Debug = "ignore")]
    pub project: Option<ProjectId>,
    #[serde(rename = "https://haste.health/user_role")]
    #[derivative(Debug = "ignore")]
    pub user_role: UserRole,
    #[serde(rename = "https://haste.health/user_id")]
    #[derivative(Debug = "ignore")]
    pub user_id: AuthorId,
    #[serde(rename = "https://haste.health/resource_type")]
    #[derivative(Debug = "ignore")]
    pub resource_type: AuthorKind,
    #[serde(rename = "https://haste.health/access_policies")]
    #[derivative(Debug = "ignore")]
    pub access_policy_version_ids: Vec<VersionId>,
    #[serde(rename = "https://haste.health/membership")]
    #[derivative(Debug = "ignore")]
    pub membership: Option<String>,
}
