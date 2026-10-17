//! For meta resources that require compute intensive operations we want to be
//! able to limit access based on the users subscription tier. This middleware
//! enforces those limits.
use crate::fhir_client::{
    ClientState, ServerCTX,
    middleware::{Backend, Middleware, Next, Output},
    subscription_limits::resource_limits::{TenantResourceLimit, get_tenant_resource_limits},
    utilities::request_to_resource_type,
};
use haste_fhir_client::request::FHIRRequest;
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::claims::SubscriptionTier;
use std::sync::Arc;

pub struct TenantTierLimits;

fn get_request_limits(
    subscription_tier: &SubscriptionTier,
    request: &FHIRRequest,
) -> Result<Vec<TenantResourceLimit>, OperationOutcomeError> {
    match request {
        FHIRRequest::Update(_) | FHIRRequest::Create(_) => {
            let Some(resource_type) = request_to_resource_type(request) else {
                return Err(OperationOutcomeError::fatal(
                    IssueType::exception(),
                    "Unable to determine resource type for request".to_string(),
                ));
            };

            Ok(get_tenant_resource_limits(subscription_tier, resource_type))
        }

        _ => Ok(vec![TenantResourceLimit::Unlimited]),
    }
}

impl<B: Backend> Middleware<B> for TenantTierLimits {
    fn call<'a>(
        &'a self,
        state: &'a ClientState<B>,
        ctx: Arc<ServerCTX<B>>,
        request: FHIRRequest,
        next: Next<'a, B>,
    ) -> Output<'a> {
        Box::pin(async move {
            let request_limits = get_request_limits(&ctx.user.claims.subscription_tier, &request)?;

            for request_limit in request_limits {
                let TenantResourceLimit::Count {
                    resource_type,
                    limit,
                } = request_limit
                else {
                    continue;
                };

                let parameters = "?_total=accurate".try_into().map_err(|e| {
                    tracing::error!(
                        "Failed to construct search query for subscription tier limit middleware: {}",
                        e
                    );

                    OperationOutcomeError::fatal(
                        IssueType::exception(),
                        "Failed to construct search query for subscription tier limit middleware"
                            .to_string(),
                    )
                })?;

                // `None` is the tenant's total footprint, counted across every
                // resource type; `Some` is the cap on one type.
                let result = match resource_type.as_ref() {
                    Some(resource_type) => {
                        ctx.client
                            .search_type(ctx.clone(), resource_type.clone(), parameters)
                            .await?
                    }
                    None => ctx.client.search_system(ctx.clone(), parameters).await?,
                };

                let total = result.total.and_then(|total| total.value).ok_or_else(|| {
                    OperationOutcomeError::fatal(
                        IssueType::exception(),
                        "Failed to retrieve total count for resource type".to_string(),
                    )
                })?;

                if total >= limit {
                    let scope = match resource_type.as_ref() {
                        Some(resource_type) => {
                            format!("resource type '{}'", resource_type.as_ref())
                        }
                        None => "stored resources".to_string(),
                    };

                    return Err(OperationOutcomeError::error(
                        IssueType::too_costly(),
                        format!(
                            "Request exceeds the limit of '{}' for {} for subscription tier {:?}",
                            limit, scope, ctx.user.claims.subscription_tier
                        ),
                    ));
                }
            }

            next.run(state, ctx, request).await
        })
    }
}
