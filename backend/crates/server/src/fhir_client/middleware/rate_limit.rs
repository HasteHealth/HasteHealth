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

// Format is read,write,search,invocation
static DEFAULT_POINT_SCORING: &str = "10,50,10,10";

static OPERATION_POINTS: LazyLock<OperationScoringPoints> = LazyLock::new(|| {
    let config = get_config(ConfigType::Environment);
    let scoring_points = config
        .get(ServerEnvironmentVariables::RateLimitOperationPoints)
        .unwrap_or(DEFAULT_POINT_SCORING.to_string());

    let format_error_message = "FORMAT ERROR: Rate limit operation points must be in the format read,write,search,invocation where each is a positive integer";

    let scoring = scoring_points
        .split(',')
        .map(|s| s.trim().parse::<u32>().expect(format_error_message))
        .collect::<Vec<u32>>();

    OperationScoringPoints {
        read: scoring.get(0).expect(format_error_message).to_owned(),
        write: scoring.get(1).expect(format_error_message).to_owned(),
        search: scoring.get(2).expect(format_error_message).to_owned(),
        invocation: scoring.get(3).expect(format_error_message).to_owned(),
    }
});

static DAY_IN_SECONDS: u32 = 60 * 60 * 24; // 1 day in seconds

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
                            DAY_IN_SECONDS as i32,
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
