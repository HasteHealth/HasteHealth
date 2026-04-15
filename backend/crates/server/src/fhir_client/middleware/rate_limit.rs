use crate::fhir_client::{
    ServerCTX,
    middleware::{ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput},
    subscription_limits::rate_limits::{
        RATE_LIMIT_WINDOW_IN_SECONDS, get_total_rate_limit_for_tier, points_for_operation,
    },
};
use haste_fhir_client::{
    FHIRClient,
    middleware::MiddlewareChain,
    request::{FHIRRequest, FHIRResponse},
};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::claims::SubscriptionTier;
use haste_rate_limit::RateLimitError;
use std::sync::Arc;

pub struct Middleware {}
impl Middleware {
    pub fn new() -> Self {
        Middleware {}
    }
}

impl<
    State: Send + Sync + 'static,
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
> MiddlewareChain<State, Arc<ServerCTX<Client>>, FHIRRequest, FHIRResponse, OperationOutcomeError>
    for Middleware
{
    fn call(
        &self,
        state: State,
        context: ServerMiddlewareContext<Client>,
        next: Option<Arc<ServerMiddlewareNext<Client, State>>>,
    ) -> ServerMiddlewareOutput<Client> {
        Box::pin(async move {
            // let start = Instant::now();
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

            // println!("Rate limit check took {} ms", start.elapsed().as_millis());

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
