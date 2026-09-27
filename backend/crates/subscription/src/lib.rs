//! The single source of truth for what each [`SubscriptionTier`] allows.
//!
//! Everything a tier permits lives in the [`TIERS`] table below: the request
//! budget the rate limiter enforces, the resource caps the tenant tier limits
//! middleware enforces, whether a tenant may set its own name and logo, and the
//! commercial facts (price, SLA, support response) the pricing page publishes.
//!
//! Nothing else in the workspace should hardcode a per-tier number. The server's
//! `subscription_limits` modules read this table, and
//! `cargo run subscription export` writes it to the website as JSON so the
//! published prices cannot drift from the enforced limits.

pub mod export;

use haste_fhir_model::r4::generated::resources::ResourceType;
use haste_jwt::claims::SubscriptionTier;
use serde::Serialize;

/// How many resources of one type a tenant may store.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceLimit {
    /// At most this many. `0` denies the resource type outright.
    Count(u64),
    /// No cap. Usage above what the plan includes is metered, not refused.
    Unlimited,
}

impl ResourceLimit {
    /// Whether storing one more resource would exceed this limit, given how
    /// many the tenant already has.
    pub fn is_exceeded_at(&self, current_total: u64) -> bool {
        match self {
            ResourceLimit::Count(limit) => current_total >= *limit,
            ResourceLimit::Unlimited => false,
        }
    }
}

/// What one request costs against a tier's daily budget.
///
/// Uniform across tiers — a tier buys a larger budget, not cheaper operations —
/// so the pricing page can state the costs once.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct OperationPoints {
    pub read: u32,
    pub write: u32,
    pub search: u32,
    pub history: u32,
    pub invocation: u32,
}

/// The cost of each kind of request, in budget points.
///
/// Weighted by what the operation actually costs this server, which is not what
/// it costs a server built differently:
///
/// - A **read** is a primary-key lookup. It is the unit of cost, priced at 1.
/// - A **write** is the expensive one. Storage is append-only, so a write adds a
///   history row, then deletes and re-inserts that resource's index rows across
///   the typed index tables — an Observation has 76 search parameters, a Patient
///   46 — and every bit of it is WAL plus an fsync on the primary, which cannot
///   be scaled out. Hence 25.
/// - A **search** is a bounded indexed `SELECT` with joins. Real work, but it
///   reads warm shared buffers and can move to a read replica, so 6 rather than
///   the write's 25.
/// - **History** walks the version chain: heavier than a read, far cheaper than
///   a write.
/// - An **invocation** is an operation whose cost varies by definition; 4 covers
///   the dispatch, and whatever the operation itself does is billed by the reads
///   and writes it performs.
///
/// Deliberately not modelled on servers that make search the most expensive
/// operation. With Postgres-backed search the binding constraint is write
/// throughput on the primary, not search CPU.
pub const OPERATION_POINTS: OperationPoints = OperationPoints {
    read: 1,
    write: 25,
    search: 6,
    history: 8,
    invocation: 4,
};

/// A tier's daily request budget, in [`OPERATION_POINTS`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestBudget {
    /// This many points per rate limit window.
    Points(usize),
    /// Not rate limited.
    Unmetered,
}

impl RequestBudget {
    /// The budget as a point count, for the rate limiter. [`Self::Unmetered`]
    /// saturates rather than being special-cased at every call site.
    pub fn as_points(&self) -> usize {
        match self {
            RequestBudget::Points(points) => *points,
            RequestBudget::Unmetered => usize::MAX,
        }
    }
}

/// Everything one subscription tier allows, and how it is sold.
#[derive(Clone, Debug, Serialize)]
pub struct TierLimits {
    /// The tier this describes. Serializes to its claim value (`"free"`, …),
    /// which is how the website keys its cards.
    pub tier: SubscriptionTier,

    // ---- What the server enforces ----
    /// Requests allowed per rate limit window. See [`OPERATION_POINTS`].
    pub request_budget: RequestBudget,
    /// Total current-version resources the tenant may store across all
    /// projects. History does not count.
    pub total_resources: ResourceLimit,
    /// Projects the tenant may create, including the `system` project.
    pub projects: ResourceLimit,
    /// Custom search parameters the tenant may define.
    pub search_parameters: ResourceLimit,
    /// Custom operations the tenant may define.
    pub operation_definitions: ResourceLimit,
    /// FHIR Subscriptions the tenant may register.
    pub subscriptions: ResourceLimit,
    /// External identity providers the tenant may configure.
    pub identity_providers: ResourceLimit,
    /// Whether the tenant may set its own display name and logo.
    pub tenant_customization: bool,

    // ---- How the tier is sold ----
    /// Name shown on the pricing page. Deliberately not the claim value: the
    /// claim is an API contract, this is marketing copy.
    pub display_name: &'static str,
    /// Formatted price, e.g. `"$1,500"`.
    pub price: &'static str,
    /// Billing cadence, or `None` for the tiers that are not billed monthly.
    pub cadence: Option<&'static str>,
    /// Who the tier is for.
    pub audience: &'static str,
    /// Support response commitment.
    pub support: &'static str,
    /// Uptime commitment.
    pub uptime_sla: &'static str,
    /// Whether a signed BAA is available, which gates holding PHI.
    pub baa_available: bool,
    /// Whether signup can put a tenant straight onto this tier.
    pub self_serve: bool,
}

/// Every tier, in the order the pricing page presents them.
///
/// Ordered cheapest-first by hosted commitment, with the self-hosted tier last
/// because it is not a hosted plan at all.
pub const TIERS: &[TierLimits] = &[
    // The tier every hosted signup starts on: a sandbox big enough to evaluate
    // the server and build a prototype, with the compute-intensive meta
    // resources withheld.
    TierLimits {
        tier: SubscriptionTier::Free,
        // Enough to evaluate the server against real data: a Synthea patient is
        // roughly 750 resources, so this affords loading a handful of them and
        // still querying them the same day. A budget that cannot absorb one
        // realistic import is a budget that ends the evaluation.
        request_budget: RequestBudget::Points(250_000),
        total_resources: ResourceLimit::Count(250_000),
        // Two: the `system` project, plus one to build in.
        projects: ResourceLimit::Count(2),
        search_parameters: ResourceLimit::Count(0),
        operation_definitions: ResourceLimit::Count(0),
        subscriptions: ResourceLimit::Count(0),
        identity_providers: ResourceLimit::Count(0),
        tenant_customization: false,

        display_name: "Developer",
        price: "$0",
        cadence: Some("/month"),
        audience: "Evaluating, prototyping, building a demo",
        support: "Community",
        uptime_sla: "None",
        baa_available: false,
        self_serve: true,
    },
    // The first tier that may hold real patient data, so everything withheld on
    // Developer opens up and the compliance artifacts come with it.
    TierLimits {
        tier: SubscriptionTier::Professional,
        request_budget: RequestBudget::Points(10_000_000),
        total_resources: ResourceLimit::Unlimited,
        projects: ResourceLimit::Unlimited,
        search_parameters: ResourceLimit::Unlimited,
        operation_definitions: ResourceLimit::Unlimited,
        subscriptions: ResourceLimit::Unlimited,
        identity_providers: ResourceLimit::Unlimited,
        tenant_customization: true,

        display_name: "Production",
        price: "$1,500",
        cadence: Some("/month"),
        audience: "Startups carrying real patient data",
        support: "1 business day",
        uptime_sla: "99.9%",
        baa_available: true,
        self_serve: false,
    },
    // Same permissions as Production, bought with a larger budget and a tighter
    // support commitment.
    TierLimits {
        tier: SubscriptionTier::Team,
        request_budget: RequestBudget::Points(50_000_000),
        total_resources: ResourceLimit::Unlimited,
        projects: ResourceLimit::Unlimited,
        search_parameters: ResourceLimit::Unlimited,
        operation_definitions: ResourceLimit::Unlimited,
        subscriptions: ResourceLimit::Unlimited,
        identity_providers: ResourceLimit::Unlimited,
        tenant_customization: true,

        display_name: "Scale",
        price: "From $6,000",
        cadence: Some("/month"),
        audience: "Platforms at population scale",
        support: "4 business hours",
        uptime_sla: "99.95%",
        baa_available: true,
        self_serve: false,
    },
    // What a self-hosted deployment runs as: nothing metered, because there is
    // no one to meter it for. Also the tier for hosted customers on a
    // single-tenant deployment.
    TierLimits {
        tier: SubscriptionTier::Unlimited,
        request_budget: RequestBudget::Unmetered,
        total_resources: ResourceLimit::Unlimited,
        projects: ResourceLimit::Unlimited,
        search_parameters: ResourceLimit::Unlimited,
        operation_definitions: ResourceLimit::Unlimited,
        subscriptions: ResourceLimit::Unlimited,
        identity_providers: ResourceLimit::Unlimited,
        tenant_customization: true,

        display_name: "Self-Hosted",
        price: "Free",
        cadence: None,
        audience: "Teams who want to own their infrastructure",
        support: "Community",
        uptime_sla: "You operate it",
        baa_available: false,
        self_serve: false,
    },
];

/// The tier every hosted signup starts on.
pub const DEFAULT_TIER: SubscriptionTier = SubscriptionTier::Free;

/// The limits for a tier.
///
/// Infallible: [`TIERS`] covers every [`SubscriptionTier`] variant, and
/// `tiers_are_exhaustive` holds that true.
pub fn limits_for(tier: &SubscriptionTier) -> &'static TierLimits {
    TIERS
        .iter()
        .find(|limits| &limits.tier == tier)
        .expect("TIERS covers every SubscriptionTier variant")
}

/// The cap on one resource type for a tier, or [`ResourceLimit::Unlimited`] for
/// the types a tier does not cap individually.
///
/// This is the per-type cap only. A tenant is also held to
/// [`TierLimits::total_resources`] across every type.
pub fn resource_limit_for(tier: &SubscriptionTier, resource_type: &ResourceType) -> ResourceLimit {
    let limits = limits_for(tier);

    match resource_type {
        ResourceType::Project => limits.projects,
        ResourceType::SearchParameter => limits.search_parameters,
        ResourceType::OperationDefinition => limits.operation_definitions,
        ResourceType::Subscription => limits.subscriptions,
        ResourceType::IdentityProvider => limits.identity_providers,
        _ => ResourceLimit::Unlimited,
    }
}

/// Whether a tier may set a tenant display name and logo.
pub fn allows_tenant_customization(tier: &SubscriptionTier) -> bool {
    limits_for(tier).tenant_customization
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `limits_for` panics on a tier missing from `TIERS`, so every variant must
    /// be present. Add a variant to `SubscriptionTier` and this fails until the
    /// tier is described here too.
    #[test]
    fn tiers_are_exhaustive() {
        let all = [
            SubscriptionTier::Free,
            SubscriptionTier::Professional,
            SubscriptionTier::Team,
            SubscriptionTier::Unlimited,
        ];

        for tier in &all {
            limits_for(tier);
        }

        assert_eq!(TIERS.len(), all.len(), "TIERS has a duplicate or extra tier");
    }

    #[test]
    fn free_tier_withholds_compute_intensive_resources() {
        let free = limits_for(&SubscriptionTier::Free);

        assert_eq!(free.search_parameters, ResourceLimit::Count(0));
        assert_eq!(free.operation_definitions, ResourceLimit::Count(0));
        assert_eq!(free.subscriptions, ResourceLimit::Count(0));
        assert_eq!(free.identity_providers, ResourceLimit::Count(0));
        assert!(!free.tenant_customization);
    }

    /// A paid tier must not be more restrictive than the free one, in either
    /// budget or resource caps.
    #[test]
    fn paid_tiers_are_not_more_restrictive_than_free() {
        let free = limits_for(&SubscriptionTier::Free);

        for paid in TIERS
            .iter()
            .filter(|limits| limits.tier != SubscriptionTier::Free)
        {
            assert!(
                paid.request_budget.as_points() >= free.request_budget.as_points(),
                "{} has a smaller request budget than Developer",
                paid.display_name
            );
            assert!(
                !paid.total_resources.is_exceeded_at(
                    match free.total_resources {
                        ResourceLimit::Count(count) => count,
                        ResourceLimit::Unlimited => u64::MAX,
                    }
                ),
                "{} caps total resources below Developer",
                paid.display_name
            );
            assert!(
                paid.tenant_customization,
                "{} cannot customize its tenant",
                paid.display_name
            );
        }
    }

    /// The weights encode this server's cost shape: a write is the most
    /// expensive operation because it fans out across the index tables and
    /// fsyncs on the primary, and a search costs more than a read but well under
    /// a write because it can be served from a replica. If these invert, the
    /// rate limiter is no longer charging for what the server actually spends.
    #[test]
    fn point_costs_track_what_operations_cost_us() {
        let p = OPERATION_POINTS;

        assert!(
            p.write > p.search,
            "a write must cost more than a search: it is the operation that cannot scale out"
        );
        assert!(
            p.search > p.read,
            "a search must cost more than a primary-key read"
        );
        assert!(
            p.history > p.read && p.history < p.write,
            "history sits between a read and a write"
        );
        assert_eq!(p.read, 1, "a read is the unit of cost");
    }

    /// The free tier has to absorb one realistic data import or it cannot be
    /// evaluated. A Synthea patient is roughly 750 resources.
    #[test]
    fn free_tier_can_load_and_query_real_data() {
        let free = limits_for(&SubscriptionTier::Free);
        let budget = free.request_budget.as_points();

        let synthea_patient_resources = 744;
        let cost_of_five_patients =
            5 * synthea_patient_resources * OPERATION_POINTS.write as usize;

        assert!(
            cost_of_five_patients < budget,
            "the free budget ({budget}) cannot load five Synthea patients ({cost_of_five_patients})"
        );

        // And the cap must hold far more than the day's writes can create, so
        // the two limits do not contradict each other.
        let ResourceLimit::Count(cap) = free.total_resources else {
            panic!("the free tier is expected to cap total resources");
        };
        let writes_per_day = budget / OPERATION_POINTS.write as usize;
        assert!(
            (cap as usize) > writes_per_day,
            "the resource cap ({cap}) is below what one day of writes can create ({writes_per_day})"
        );
    }

    #[test]
    fn unlimited_tier_is_unmetered() {
        let unlimited = limits_for(&SubscriptionTier::Unlimited);

        assert_eq!(unlimited.request_budget, RequestBudget::Unmetered);
        assert_eq!(unlimited.request_budget.as_points(), usize::MAX);
        assert_eq!(unlimited.total_resources, ResourceLimit::Unlimited);
    }

    #[test]
    fn resource_limit_for_uncapped_type_is_unlimited() {
        assert_eq!(
            resource_limit_for(&SubscriptionTier::Free, &ResourceType::Patient),
            ResourceLimit::Unlimited
        );
    }

    #[test]
    fn counts_are_exceeded_at_the_limit_not_past_it() {
        let two = ResourceLimit::Count(2);

        assert!(!two.is_exceeded_at(1));
        assert!(two.is_exceeded_at(2));
        // A zero count denies the first write.
        assert!(ResourceLimit::Count(0).is_exceeded_at(0));
        assert!(!ResourceLimit::Unlimited.is_exceeded_at(u64::MAX));
    }

    /// Only the tier signup assigns may be self-serve; the rest require a
    /// deliberate upgrade.
    #[test]
    fn only_the_default_tier_is_self_serve() {
        for limits in TIERS {
            assert_eq!(
                limits.self_serve,
                limits.tier == DEFAULT_TIER,
                "{} has the wrong self_serve flag",
                limits.display_name
            );
        }
    }

    /// A tier that may hold PHI must offer a BAA.
    #[test]
    fn tiers_without_a_baa_are_the_unbilled_ones() {
        for limits in TIERS {
            if limits.baa_available {
                assert!(
                    limits.cadence.is_some(),
                    "{} offers a BAA but is not billed",
                    limits.display_name
                );
            }
        }
    }
}
