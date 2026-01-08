#![allow(dead_code)]
use haste_fhir_client::request::FHIRRequest;
use haste_fhir_model::r4::generated::{resources::Bundle, terminology::HttpVerb};
use haste_jwt::claims::SubscriptionTier;

static INVOCATION_POINTS: u32 = 100;
static WRITE_POINTS: u32 = 100;
static SEARCH_POINTS: u32 = 30;
static READ_POINTS: u32 = 10;

// Per day Limits
static FREE_TIER: u32 = 25000;
static PRO_TIER: u32 = 1000000;
static TEAM_TIER: u32 = 5000000;
static UNLIMITED_TIER: u32 = u32::MAX;

pub fn get_total_rate_limit_for_tier(tier: &SubscriptionTier) -> u32 {
    match tier {
        SubscriptionTier::Free => FREE_TIER,
        SubscriptionTier::Professional => PRO_TIER,
        SubscriptionTier::Team => TEAM_TIER,
        SubscriptionTier::Unlimited => UNLIMITED_TIER,
    }
}

fn score_bundle(bundle: &Bundle) -> u32 {
    let mut total_points: u32 = 0;

    let default = vec![];
    for entry in bundle.entry.as_ref().unwrap_or(&default).iter() {
        let method = entry.request.as_ref().map(|req| req.method.as_ref());

        match method.unwrap_or(&HttpVerb::Null(None)) {
            HttpVerb::PATCH(_) | HttpVerb::PUT(_) | HttpVerb::POST(_) | HttpVerb::DELETE(_) => {
                total_points += WRITE_POINTS
            }
            HttpVerb::GET(_) => total_points += SEARCH_POINTS,
            HttpVerb::Null(_) | HttpVerb::HEAD(_) => {
                // Do nothing for null/head
            }
        }
    }

    total_points
}

pub fn points_for_operation(request: &FHIRRequest) -> u32 {
    match request {
        FHIRRequest::Read(_) => READ_POINTS,
        FHIRRequest::VersionRead(_) => READ_POINTS,

        FHIRRequest::Create(_) => WRITE_POINTS,
        FHIRRequest::Update(_) => WRITE_POINTS,
        FHIRRequest::Patch(_) => WRITE_POINTS,
        FHIRRequest::Delete(_) => WRITE_POINTS,

        FHIRRequest::Capabilities => 10,
        FHIRRequest::Search(_) => SEARCH_POINTS,
        FHIRRequest::History(_) => SEARCH_POINTS,

        FHIRRequest::Invocation(_) => INVOCATION_POINTS,

        FHIRRequest::Batch(fhirbatch_request) => score_bundle(&fhirbatch_request.resource),
        FHIRRequest::Transaction(fhirtransaction_request) => {
            score_bundle(&fhirtransaction_request.resource)
        }
    }
}
