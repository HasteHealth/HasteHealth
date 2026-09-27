//! Per-tier resource caps. The numbers live in the `haste-subscription` crate;
//! this decides which cap a given request is held to.

use haste_fhir_model::r4::generated::resources::ResourceType;
use haste_jwt::claims::SubscriptionTier;
use haste_subscription::ResourceLimit;

/// The cap a write is held to, and the search that counts against it.
pub enum TenantResourceLimit {
    /// At most `limit` resources of `resource_type`. Counting `None` means
    /// counting every resource type — the tenant's total footprint.
    Count {
        resource_type: Option<ResourceType>,
        limit: u64,
    },
    Unlimited,
}

/// The caps that apply to creating a resource of this type, tightest first.
///
/// A write can be refused by either the cap on its own resource type or the
/// tenant's total resource cap, so both are returned and the middleware checks
/// them in order. The per-type cap goes first: it is the cheaper count and the
/// clearer error message.
pub fn get_tenant_resource_limits(
    tier: &SubscriptionTier,
    resource_type: &ResourceType,
) -> Vec<TenantResourceLimit> {
    let mut limits = Vec::with_capacity(2);

    if let ResourceLimit::Count(limit) = haste_subscription::resource_limit_for(tier, resource_type)
    {
        limits.push(TenantResourceLimit::Count {
            resource_type: Some(resource_type.clone()),
            limit,
        });
    }

    if let ResourceLimit::Count(limit) = haste_subscription::limits_for(tier).total_resources {
        limits.push(TenantResourceLimit::Count {
            resource_type: None,
            limit,
        });
    }

    if limits.is_empty() {
        limits.push(TenantResourceLimit::Unlimited);
    }

    limits
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The free tier's total cap, read from the crate rather than restated, so
    /// changing the published limit does not silently break these tests.
    fn free_total_resources() -> ResourceLimit {
        haste_subscription::limits_for(&SubscriptionTier::Free).total_resources
    }

    #[test]
    fn free_tier_checks_the_type_cap_before_the_total() {
        let limits =
            get_tenant_resource_limits(&SubscriptionTier::Free, &ResourceType::SearchParameter);

        assert_eq!(limits.len(), 2);
        match &limits[0] {
            TenantResourceLimit::Count {
                resource_type,
                limit,
            } => {
                assert_eq!(resource_type.as_ref(), Some(&ResourceType::SearchParameter));
                assert_eq!(*limit, 0);
            }
            TenantResourceLimit::Unlimited => panic!("expected the SearchParameter cap"),
        }
        match &limits[1] {
            TenantResourceLimit::Count {
                resource_type,
                limit,
            } => {
                assert_eq!(*resource_type, None);
                assert_eq!(ResourceLimit::Count(*limit), free_total_resources());
            }
            TenantResourceLimit::Unlimited => panic!("expected the total resource cap"),
        }
    }

    /// An uncapped type on the free tier still counts against the total.
    #[test]
    fn free_tier_holds_uncapped_types_to_the_total() {
        let limits = get_tenant_resource_limits(&SubscriptionTier::Free, &ResourceType::Patient);

        assert_eq!(limits.len(), 1);
        match &limits[0] {
            TenantResourceLimit::Count {
                resource_type,
                limit,
            } => {
                assert_eq!(*resource_type, None);
                assert_eq!(ResourceLimit::Count(*limit), free_total_resources());
            }
            TenantResourceLimit::Unlimited => panic!("expected the total resource cap"),
        }
    }

    #[test]
    fn paid_tiers_are_uncapped() {
        for tier in [
            SubscriptionTier::Professional,
            SubscriptionTier::Team,
            SubscriptionTier::Unlimited,
        ] {
            for resource_type in [ResourceType::Patient, ResourceType::SearchParameter] {
                let limits = get_tenant_resource_limits(&tier, &resource_type);

                assert_eq!(limits.len(), 1);
                assert!(matches!(limits[0], TenantResourceLimit::Unlimited));
            }
        }
    }
}
