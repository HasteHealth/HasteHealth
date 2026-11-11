use crate::fhir_client::{
    ClientState, ServerCTX,
    middleware::{Backend, Middleware, Next, Output},
    subscription_limits::rate_limits::{get_total_rate_limit_for_tier, points_for_operation},
};
use chrono::{SecondsFormat, TimeDelta, Utc};
use haste_fhir_client::request::FHIRRequest;
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::claims::SubscriptionTier;
use haste_rate_limit::RateLimitError;
use std::{sync::Arc, time::Duration};

pub struct RateLimit;

/// `"45 seconds"`, `"12 minutes"`, `"2 hours"`: a duration rounded up to its
/// largest whole unit, so a caller that waits as long as it says is never early.
fn describe_duration(duration: Duration) -> String {
    let seconds = duration.as_secs() + u64::from(duration.subsec_nanos() > 0);
    let (count, unit) = match seconds {
        0..60 => (seconds, "second"),
        60..3600 => (seconds.div_ceil(60), "minute"),
        _ => (seconds.div_ceil(3600), "hour"),
    };

    if count == 1 {
        format!("1 {unit}")
    } else {
        format!("{count} {unit}s")
    }
}

/// Refuses a request the window has no room left for, saying when it resets.
fn exceeded(points: u32, max: usize, window: &str, retry_after: Duration) -> OperationOutcomeError {
    let resets_at = Utc::now() + TimeDelta::from_std(retry_after).unwrap_or(TimeDelta::zero());

    OperationOutcomeError::error(
        IssueType::throttled(),
        format!(
            "Rate limit exceeded: this request costs {points} points and less than that remains of the {max} this tenant may spend every {window}. The limit resets at {} (in {}).",
            resets_at.to_rfc3339_opts(SecondsFormat::Secs, true),
            describe_duration(retry_after),
        ),
    )
}

/// Refuses a request that costs more than a whole window allows. Waiting does
/// not help, so it is not answered as throttled.
fn too_costly(points: u32, max: usize, window: &str) -> OperationOutcomeError {
    let error = OperationOutcomeError::error(
        IssueType::too_costly(),
        format!(
            "This request costs {points} points, more than the {max} this tenant may spend every {window}, so it is refused however long you wait. Split it into smaller requests."
        ),
    );

    OperationOutcomeError::unprocessable(error.outcome().clone())
}

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
                    let window_seconds = state.config.rate_limits.rate_limit_window_seconds;
                    let window = describe_duration(Duration::from_secs(window_seconds));

                    if points as usize > max_score_for_tenant {
                        return Err(too_costly(points, max_score_for_tenant, &window));
                    }

                    ctx.rate_limit
                        .check(
                            ctx.tenant.as_ref(),
                            max_score_for_tenant as i32,
                            points as i32,
                            window_seconds as i32,
                        )
                        .await
                        .map_err(|e| match e {
                            RateLimitError::Exceeded { retry_after } => {
                                exceeded(points, max_score_for_tenant, &window, retry_after)
                            }
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Rounded up, so a caller told to wait that long is never early.
    #[test]
    fn describes_durations_rounded_up() {
        assert_eq!(describe_duration(Duration::from_secs(1)), "1 second");
        assert_eq!(
            describe_duration(Duration::from_millis(44_200)),
            "45 seconds"
        );
        assert_eq!(describe_duration(Duration::from_secs(60)), "1 minute");
        assert_eq!(describe_duration(Duration::from_secs(61)), "2 minutes");
        assert_eq!(
            describe_duration(Duration::from_secs(15 * 60)),
            "15 minutes"
        );
        assert_eq!(
            describe_duration(Duration::from_secs(24 * 3600)),
            "24 hours"
        );
    }

    /// The client is told when it may retry, in the outcome it already prints.
    #[test]
    fn exceeded_says_when_the_limit_resets() {
        let error = exceeded(18_750, 500_000, "15 minutes", Duration::from_secs(600));
        let diagnostics = error.outcome().issue[0]
            .diagnostics
            .as_ref()
            .and_then(|d| d.value.clone())
            .expect("the refusal has diagnostics");

        assert_eq!(error.http_status(), 429);
        assert!(diagnostics.contains("costs 18750 points"), "{diagnostics}");
        assert!(diagnostics.contains("every 15 minutes"), "{diagnostics}");
        assert!(
            diagnostics.contains("The limit resets at "),
            "{diagnostics}"
        );
        assert!(diagnostics.ends_with("(in 10 minutes)."), "{diagnostics}");
    }

    /// A request bigger than a whole window must not tell the client to retry.
    #[test]
    fn too_costly_is_not_throttled() {
        let error = too_costly(600_000, 500_000, "15 minutes");

        assert_eq!(error.http_status(), 422);
        assert_eq!(error.outcome().issue[0].code.as_str(), Some("too-costly"));
    }
}
