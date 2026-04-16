use std::{collections::HashMap, sync::LazyLock};

use haste_fhir_model::r4::generated::resources::ResourceType;
use haste_jwt::claims::SubscriptionTier;

/// Hardcoding limits for now

#[derive(Clone)]
pub enum TenantResourceLimit {
    Count(ResourceType, usize),
    Unlimited,
}

static FREE_TIER_OPERATION_LIMITS: usize = 0;
static FREE_TIER_SUBSCRIPTION_LIMITS: usize = 0;
static FREE_TIER_SEARCH_PARAMETER_LIMITS: usize = 0;

static SUBSCRIPTION_LIMITS: LazyLock<
    HashMap<SubscriptionTier, HashMap<ResourceType, TenantResourceLimit>>,
> = LazyLock::new(|| {
    let mut limits = HashMap::new();

    let mut free_tier_limits = HashMap::new();
    free_tier_limits.insert(
        ResourceType::OperationDefinition,
        TenantResourceLimit::Count(
            ResourceType::OperationDefinition,
            FREE_TIER_OPERATION_LIMITS,
        ),
    );
    free_tier_limits.insert(
        ResourceType::Subscription,
        TenantResourceLimit::Count(ResourceType::Subscription, FREE_TIER_SUBSCRIPTION_LIMITS),
    );
    free_tier_limits.insert(
        ResourceType::SearchParameter,
        TenantResourceLimit::Count(
            ResourceType::SearchParameter,
            FREE_TIER_SEARCH_PARAMETER_LIMITS,
        ),
    );

    limits.insert(SubscriptionTier::Free, free_tier_limits);

    limits
});

pub fn get_tenant_resource_limit(
    tier: &SubscriptionTier,
    resource_type: &ResourceType,
) -> TenantResourceLimit {
    SUBSCRIPTION_LIMITS
        .get(tier)
        .and_then(|resource_limits| resource_limits.get(resource_type).cloned())
        .unwrap_or(TenantResourceLimit::Unlimited)
}
