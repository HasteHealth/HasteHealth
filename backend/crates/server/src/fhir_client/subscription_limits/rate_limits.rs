//! Scores a request against a tier's budget. The point costs and per-tier
//! budgets live in the `haste-subscription` crate; a deployment may override the
//! budgets through `rate_limit_subscription_tiers` in its config.

use crate::config::ServerConfig;

use haste_fhir_client::request::FHIRRequest;
use haste_fhir_model::r4::generated::{resources::Bundle, terminology::HttpVerb};
use haste_jwt::claims::SubscriptionTier;
use haste_subscription::{OPERATION_POINTS, TIERS};
use std::sync::OnceLock;

/// Each tier's budget, in the order [`TIERS`] declares them, after applying any
/// config override.
static SUBSCRIPTION_TIERS: OnceLock<Vec<usize>> = OnceLock::new();

/// The published budgets, with `rate_limit_subscription_tiers` overriding them
/// position by position when configured.
fn setup_subscription_tiers(config: &ServerConfig) -> Vec<usize> {
    let overrides = config.rate_limits.rate_limit_subscription_tiers.as_ref();

    TIERS
        .iter()
        .enumerate()
        .map(|(index, limits)| {
            overrides
                .and_then(|tiers| tiers.get(index).copied())
                .unwrap_or_else(|| limits.request_budget.as_points())
        })
        .collect()
}

pub fn get_total_rate_limit_for_tier(config: &ServerConfig, tier: &SubscriptionTier) -> usize {
    let tiers = SUBSCRIPTION_TIERS.get_or_init(|| setup_subscription_tiers(config));

    TIERS
        .iter()
        .position(|limits| &limits.tier == tier)
        .and_then(|index| tiers.get(index).copied())
        .unwrap_or_else(|| haste_subscription::limits_for(tier).request_budget.as_points())
}

fn score_bundle(bundle: &Bundle) -> u32 {
    let mut total_points: u32 = 0;

    let default = vec![];
    for entry in bundle.entry.as_ref().unwrap_or(&default).iter() {
        let method = entry.request.as_ref().map(|req| &req.method);

        match method.unwrap_or(&HttpVerb::null()) {
            method
                if method == &HttpVerb::patch()
                    || method == &HttpVerb::put()
                    || method == &HttpVerb::post()
                    || method == &HttpVerb::delete() =>
            {
                total_points += OPERATION_POINTS.write
            }
            // A GET entry is a search only when it carries a query; otherwise it
            // is a read by id. Charging every GET as a search overcharges a
            // bundled read six-fold now that the weights differ.
            method if method == &HttpVerb::get() => {
                let is_search = entry
                    .request
                    .as_ref()
                    .and_then(|req| req.url.value.as_deref())
                    .is_some_and(|url| url.contains('?'));

                total_points += if is_search {
                    OPERATION_POINTS.search
                } else {
                    OPERATION_POINTS.read
                };
            }
            method if method == &HttpVerb::null() || method == &HttpVerb::head() => {
                // Do nothing for null/head
            }
            _ => {
                // do nothing.
            }
        }
    }

    total_points
}

pub fn points_for_operation(config: &ServerConfig, request: &FHIRRequest) -> u32 {
    match request {
        FHIRRequest::Read(_) => OPERATION_POINTS.read,
        FHIRRequest::VersionRead(_) => OPERATION_POINTS.read,

        FHIRRequest::Create(_) => OPERATION_POINTS.write,
        FHIRRequest::Update(_) => OPERATION_POINTS.write,
        FHIRRequest::Patch(_) => OPERATION_POINTS.write,
        FHIRRequest::Delete(_) => OPERATION_POINTS.write,

        FHIRRequest::Capabilities => OPERATION_POINTS.invocation,
        FHIRRequest::Search(_) => OPERATION_POINTS.search,
        FHIRRequest::History(_) => OPERATION_POINTS.history,

        FHIRRequest::Invocation(_) => OPERATION_POINTS.invocation,

        FHIRRequest::Batch(fhirbatch_request) => score_bundle(&fhirbatch_request.resource),
        FHIRRequest::Transaction(fhirtransaction_request) => {
            score_bundle(&fhirtransaction_request.resource)
        }
        FHIRRequest::Compartment(compartment_request) => {
            points_for_operation(config, &compartment_request.request)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use haste_fhir_model::r4::generated::{
        resources::{BundleEntry, BundleEntryRequest},
        terminology::BoundCode,
    };

    fn entry(method: BoundCode<HttpVerb>, url: &str) -> BundleEntry {
        BundleEntry {
            request: Some(BundleEntryRequest {
                method,
                url: Box::new(url.to_string().into()),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn bundle(entries: Vec<BundleEntry>) -> Bundle {
        Bundle {
            entry: Some(entries),
            ..Default::default()
        }
    }

    /// A bundled GET with a query is a search; one without is a read by id.
    /// Charging both as searches overcharges a bundled read six-fold.
    #[test]
    fn scores_bundled_reads_and_searches_differently() {
        let reads = bundle(vec![entry(HttpVerb::get(), "Patient/123")]);
        assert_eq!(score_bundle(&reads), OPERATION_POINTS.read);

        let searches = bundle(vec![entry(HttpVerb::get(), "Patient?name=smith")]);
        assert_eq!(score_bundle(&searches), OPERATION_POINTS.search);
    }

    #[test]
    fn scores_bundled_writes_as_writes() {
        for method in [
            HttpVerb::post(),
            HttpVerb::put(),
            HttpVerb::patch(),
            HttpVerb::delete(),
        ] {
            let b = bundle(vec![entry(method, "Patient/123")]);
            assert_eq!(score_bundle(&b), OPERATION_POINTS.write);
        }
    }

    /// The whole point of a weighted bundle score: cost is the sum of entries.
    #[test]
    fn scores_a_mixed_bundle_as_the_sum_of_its_entries() {
        let b = bundle(vec![
            entry(HttpVerb::get(), "Patient/1"),
            entry(HttpVerb::get(), "Observation?patient=1"),
            entry(HttpVerb::post(), "Observation"),
        ]);

        assert_eq!(
            score_bundle(&b),
            OPERATION_POINTS.read + OPERATION_POINTS.search + OPERATION_POINTS.write
        );
    }

    /// With no override, every tier gets the budget the pricing page publishes.
    #[test]
    fn defaults_to_the_published_budgets() {
        let config = ServerConfig::default();
        let tiers = setup_subscription_tiers(&config);

        for (index, limits) in TIERS.iter().enumerate() {
            assert_eq!(
                tiers[index],
                limits.request_budget.as_points(),
                "{} does not get its published budget",
                limits.display_name
            );
        }
    }

    /// The config array overrides budgets in the order `TIERS` declares them.
    #[test]
    fn config_overrides_budgets_positionally() {
        let mut config = ServerConfig::default();
        config.rate_limits.rate_limit_subscription_tiers = Some([1, 2, 3, 4]);

        let tiers = setup_subscription_tiers(&config);

        assert_eq!(tiers, vec![1, 2, 3, 4]);
    }

    /// The override array has one slot per tier. Adding a tier without widening
    /// it would leave the new tier un-overridable, so assert the widths match.
    #[test]
    fn override_array_covers_every_tier() {
        let mut config = ServerConfig::default();
        config.rate_limits.rate_limit_subscription_tiers = Some([1, 2, 3, 4]);

        let width = config
            .rate_limits
            .rate_limit_subscription_tiers
            .expect("just set")
            .len();

        assert_eq!(
            width,
            TIERS.len(),
            "rate_limit_subscription_tiers must have one entry per tier"
        );
    }

    /// Every tier resolves to its own budget, keyed by tier rather than by
    /// position at the call site.
    #[test]
    fn resolves_each_tier_to_its_own_budget() {
        let config = ServerConfig::default();

        assert_eq!(
            get_total_rate_limit_for_tier(&config, &SubscriptionTier::Free),
            250_000
        );
        assert_eq!(
            get_total_rate_limit_for_tier(&config, &SubscriptionTier::Professional),
            10_000_000
        );
        assert_eq!(
            get_total_rate_limit_for_tier(&config, &SubscriptionTier::Team),
            50_000_000
        );
        assert_eq!(
            get_total_rate_limit_for_tier(&config, &SubscriptionTier::Unlimited),
            usize::MAX
        );
    }
}
