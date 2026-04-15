use std::{collections::HashMap, sync::LazyLock};

use haste_fhir_model::r4::generated::resources::ResourceType;
use haste_jwt::claims::SubscriptionTier;

/// Hardcoding limits for now

#[derive(Clone)]
pub enum Limit {
    #[allow(dead_code)]
    Count(usize),
    Unlimited,
}

static SUBSCRIPTION_LIMITS: LazyLock<HashMap<SubscriptionTier, HashMap<ResourceType, Limit>>> =
    LazyLock::new(|| {
        let mut limits = HashMap::new();

        let mut free_tier_limits = HashMap::new();
        free_tier_limits.insert(ResourceType::OperationDefinition, Limit::Count(0));
        free_tier_limits.insert(ResourceType::Subscription, Limit::Count(0));
        free_tier_limits.insert(ResourceType::SearchParameter, Limit::Count(0));

        limits.insert(SubscriptionTier::Free, free_tier_limits);

        limits
    });

#[allow(dead_code)]
pub fn get_subscription_resource_limit(
    tier: &SubscriptionTier,
    resource_type: &ResourceType,
) -> Limit {
    SUBSCRIPTION_LIMITS
        .get(tier)
        .and_then(|resource_limits| resource_limits.get(resource_type).cloned())
        .unwrap_or(Limit::Unlimited)
}
