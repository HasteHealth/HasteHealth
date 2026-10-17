use crate::fhir_client::{
    ClientState, ServerCTX,
    middleware::{Backend, Middleware, Next, Output},
    subscription_limits::rate_limits::{get_total_rate_limit_for_tier, points_for_operation},
};
use haste_fhir_client::request::FHIRRequest;
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::claims::SubscriptionTier;
use haste_rate_limit::RateLimitError;
use std::sync::Arc;

pub struct RateLimit;

impl<B: Backend> Middleware<B> for RateLimit {
    fn call<'a>(
        &'a self,
        state: &'a ClientState<B>,
        ctx: Arc<ServerCTX<B>>,
        request: FHIRRequest,
        next: Next<'a, B>,
    ) -> Output<'a> {
        Box::pin(async move {
            match &ctx.user.claims.subscription_tier {
                SubscriptionTier::Free
                | SubscriptionTier::Professional
                | SubscriptionTier::Team => {
                    let max_score_for_tenant = get_total_rate_limit_for_tier(
                        state.config.as_ref(),
                        &ctx.user.claims.subscription_tier,
                    );
                    let points = points_for_operation(state.config.as_ref(), &request);

                    ctx.rate_limit
                        .check(
                            ctx.tenant.as_ref(),
                            max_score_for_tenant as i32,
                            points as i32,
                            state.config.rate_limits.rate_limit_window_seconds as i32,
                        )
                        .await
                        .map_err(|e| match e {
                            RateLimitError::Exceeded => OperationOutcomeError::error(
                                IssueType::throttled(),
                                "Rate limit exceeded".to_string(),
                            ),
                            RateLimitError::Error(msg) => {
                                tracing::error!("Rate limit error: {}", msg);
                                OperationOutcomeError::fatal(
                                    IssueType::exception(),
                                    "Failed to process rate limit".to_string(),
                                )
                            }
                        })?;
                }
                SubscriptionTier::Unlimited => {
                    // Do nothing for unlimited.
                }
            }

            next.run(state, ctx, request).await
        })
    }
}
