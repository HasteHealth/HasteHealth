use crate::{
    ServerEnvironmentVariables,
    fhir_client::{
        ServerCTX,
        middleware::{
            ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput,
            ServerMiddlewareState,
        },
    },
};
use haste_config::{ConfigType, get_config};
use haste_fhir_client::{
    middleware::MiddlewareChain,
    request::{FHIRRequest, FHIRResponse},
};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_model::r4::generated::{resources::Bundle, terminology::HttpVerb};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::claims::SubscriptionTier;
use haste_rate_limit::RateLimitError;
use haste_repository::Repository;
use std::sync::{Arc, LazyLock};

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

static OPERATION_POINTS: LazyLock<OperationScoringPoints> = LazyLock::new(|| {
    let config = get_config(ConfigType::Environment);
    let Ok(scoring_points) = config.get(ServerEnvironmentVariables::RateLimitOperationPoints)
    else {
        return OperationScoringPoints {
            read: DEFAULT_READ_POINTS,
            write: DEFAULT_WRITE_POINTS,
            search: DEFAULT_SEARCH_POINTS,
            invocation: DEFAULT_INVOCATION_POINTS,
        };
    };

    let format_error_message = "FORMAT ERROR: Rate limit operation points must be in the format read,write,search,invocation where each is a positive integer";

    let scoring = scoring_points
        .split(',')
        .map(|s| s.trim().parse::<u32>().expect(format_error_message))
        .collect::<Vec<u32>>();

    OperationScoringPoints {
        read: scoring.get(0).unwrap_or(&DEFAULT_READ_POINTS).to_owned(),
        write: scoring.get(1).unwrap_or(&DEFAULT_WRITE_POINTS).to_owned(),
        search: scoring.get(2).unwrap_or(&DEFAULT_SEARCH_POINTS).to_owned(),
        invocation: scoring
            .get(3)
            .unwrap_or(&DEFAULT_INVOCATION_POINTS)
            .to_owned(),
    }
});

static RATE_LIMIT_WINDOW_IN_SECONDS: LazyLock<u32> = LazyLock::new(|| {
    let config = get_config(ConfigType::Environment);
    config
        .get(ServerEnvironmentVariables::RateLimitWindowInSeconds)
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(DEFAULT_RATE_LIMIT_WINDOW)
        .to_owned()
});

static DEFAULT_RATE_LIMIT_WINDOW: u32 = 60 * 60 * 24; // 1 day in seconds

// Per day Limits
static DEFAULT_FREE_TIER: u32 = 25000;
static DEFAULT_PRO_TIER: u32 = 1000000;
static DEFAULT_TEAM_TIER: u32 = 5000000;

struct SubscriptionTiers {
    free: u32,
    professional: u32,
    team: u32,
    unlimited: u32,
}

static SUBSCRIPTION_TIERS: LazyLock<SubscriptionTiers> = LazyLock::new(|| {
    let config = get_config(ConfigType::Environment);
    let Ok(subscription_tiers_rate_limit) =
        config.get(ServerEnvironmentVariables::RateLimitSubscriptions)
    else {
        return SubscriptionTiers {
            free: DEFAULT_FREE_TIER,
            professional: DEFAULT_PRO_TIER,
            team: DEFAULT_TEAM_TIER,
            unlimited: u32::MAX,
        };
    };

    let format_error_message = "FORMAT ERROR: Rate limit subscription tiers must be in the format free,professional,team where each is a positive integer";

    let subscription_tiers = subscription_tiers_rate_limit
        .split(',')
        .map(|s| s.trim().parse::<u32>().expect(format_error_message))
        .collect::<Vec<u32>>();

    SubscriptionTiers {
        free: subscription_tiers
            .get(0)
            .unwrap_or(&DEFAULT_FREE_TIER)
            .to_owned(),
        professional: subscription_tiers
            .get(1)
            .unwrap_or(&DEFAULT_PRO_TIER)
            .to_owned(),
        team: subscription_tiers
            .get(2)
            .unwrap_or(&DEFAULT_TEAM_TIER)
            .to_owned(),
        unlimited: u32::MAX,
    }
});

pub fn get_total_rate_limit_for_tier(tier: &SubscriptionTier) -> u32 {
    match tier {
        SubscriptionTier::Free => SUBSCRIPTION_TIERS.free,
        SubscriptionTier::Professional => SUBSCRIPTION_TIERS.professional,
        SubscriptionTier::Team => SUBSCRIPTION_TIERS.team,
        SubscriptionTier::Unlimited => SUBSCRIPTION_TIERS.unlimited,
    }
}

fn score_bundle(bundle: &Bundle) -> u32 {
    let mut total_points: u32 = 0;

    let default = vec![];
    for entry in bundle.entry.as_ref().unwrap_or(&default).iter() {
        let method = entry.request.as_ref().map(|req| req.method.as_ref());

        match method.unwrap_or(&HttpVerb::Null(None)) {
            HttpVerb::PATCH(_) | HttpVerb::PUT(_) | HttpVerb::POST(_) | HttpVerb::DELETE(_) => {
                total_points += OPERATION_POINTS.write
            }
            HttpVerb::GET(_) => total_points += OPERATION_POINTS.search,
            HttpVerb::Null(_) | HttpVerb::HEAD(_) => {
                // Do nothing for null/head
            }
        }
    }

    total_points
}

fn points_for_operation(request: &FHIRRequest) -> u32 {
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
            points_for_operation(&compartment_request.request)
        }
    }
}

pub struct Middleware {}
impl Middleware {
    pub fn new() -> Self {
        Middleware {}
    }
}

impl<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>
    MiddlewareChain<
        ServerMiddlewareState<Repo, Search, Terminology>,
        Arc<ServerCTX<Repo, Search, Terminology>>,
        FHIRRequest,
        FHIRResponse,
        OperationOutcomeError,
    > for Middleware
{
    fn call(
        &self,
        state: ServerMiddlewareState<Repo, Search, Terminology>,
        context: ServerMiddlewareContext<Repo, Search, Terminology>,
        next: Option<Arc<ServerMiddlewareNext<Repo, Search, Terminology>>>,
    ) -> ServerMiddlewareOutput<Repo, Search, Terminology> {
        Box::pin(async move {
            match &context.ctx.user.subscription_tier {
                SubscriptionTier::Free
                | SubscriptionTier::Professional
                | SubscriptionTier::Team => {
                    let max_score_for_tenant =
                        get_total_rate_limit_for_tier(&context.ctx.user.subscription_tier);
                    let points = points_for_operation(&context.request);

                    context
                        .ctx
                        .rate_limit
                        .check(
                            context.ctx.tenant.as_ref(),
                            max_score_for_tenant as i32,
                            points as i32,
                            *RATE_LIMIT_WINDOW_IN_SECONDS as i32,
                        )
                        .await
                        .map_err(|e| match e {
                            RateLimitError::Exceeded => OperationOutcomeError::error(
                                IssueType::Throttled(None),
                                "Rate limit exceeded".to_string(),
                            ),
                            RateLimitError::Error(msg) => {
                                tracing::error!("Rate limit error: {}", msg);
                                OperationOutcomeError::fatal(
                                    IssueType::Exception(None),
                                    "Failed to process rate limit".to_string(),
                                )
                            }
                        })?;
                }
                SubscriptionTier::Unlimited => {
                    // Do nothing for unlimited.
                }
            }

            if let Some(next) = next {
                next(state, context).await
            } else {
                Err(OperationOutcomeError::fatal(
                    IssueType::Exception(None),
                    "No next middleware found".to_string(),
                ))
            }
        })
    }
}
