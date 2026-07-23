use crate::config::ServerConfig;

use haste_fhir_client::request::FHIRRequest;
use haste_fhir_model::r4::generated::{resources::Bundle, terminology::HttpVerb};
use haste_jwt::claims::SubscriptionTier;
use std::sync::{LazyLock, OnceLock};

struct OperationScoringPoints {
    read: u32,
    write: u32,
    search: u32,
    invocation: u32,
}

static DEFAULT_READ_POINTS: u32 = 10;
static DEFAULT_WRITE_POINTS: u32 = 50;
static DEFAULT_SEARCH_POINTS: u32 = 10;
static DEFAULT_INVOCATION_POINTS: u32 = 10;

static OPERATION_POINTS: LazyLock<OperationScoringPoints> =
    LazyLock::new(|| OperationScoringPoints {
        read: DEFAULT_READ_POINTS,
        write: DEFAULT_WRITE_POINTS,
        search: DEFAULT_SEARCH_POINTS,
        invocation: DEFAULT_INVOCATION_POINTS,
    });

// Per day Limits
static DEFAULT_FREE_TIER: usize = 25000;
static DEFAULT_PRO_TIER: usize = 1000000;
static DEFAULT_TEAM_TIER: usize = 5000000;

struct SubscriptionTiers {
    free: usize,
    professional: usize,
    team: usize,
    unlimited: usize,
}

fn setup_subscription_tiers(config: &ServerConfig) -> SubscriptionTiers {
    if let Some(subscription_tiers_rate_limit) = &config.rate_limits.rate_limit_subscription_tiers {
        SubscriptionTiers {
            free: subscription_tiers_rate_limit
                .get(0)
                .unwrap_or(&DEFAULT_FREE_TIER)
                .to_owned(),
            professional: subscription_tiers_rate_limit
                .get(1)
                .unwrap_or(&DEFAULT_PRO_TIER)
                .to_owned(),
            team: subscription_tiers_rate_limit
                .get(2)
                .unwrap_or(&DEFAULT_TEAM_TIER)
                .to_owned(),
            unlimited: usize::MAX,
        }
    } else {
        SubscriptionTiers {
            free: DEFAULT_FREE_TIER,
            professional: DEFAULT_PRO_TIER,
            team: DEFAULT_TEAM_TIER,
            unlimited: usize::MAX,
        }
    }
}

static SUBSCRIPTION_TIERS: OnceLock<SubscriptionTiers> = OnceLock::new();

pub fn get_total_rate_limit_for_tier(config: &ServerConfig, tier: &SubscriptionTier) -> usize {
    let tiers = SUBSCRIPTION_TIERS.get_or_init(|| setup_subscription_tiers(config));

    match tier {
        SubscriptionTier::Free => tiers.free,
        SubscriptionTier::Professional => tiers.professional,
        SubscriptionTier::Team => tiers.team,
        SubscriptionTier::Unlimited => tiers.unlimited,
    }
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
            method if method == &HttpVerb::get() => total_points += OPERATION_POINTS.search,
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
        FHIRRequest::History(_) => OPERATION_POINTS.search,

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
