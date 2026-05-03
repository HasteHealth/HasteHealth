/// For meta resources that require compute intensive operations we want to be able to limit access based on the users subscription tier.
///  This middleware enforces those limits.
use crate::fhir_client::{
    ServerCTX,
    middleware::{ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput},
    subscription_limits::resource_limits::{TenantResourceLimit, get_tenant_resource_limit},
    utilities::request_to_resource_type,
};
use haste_fhir_client::{
    FHIRClient,
    middleware::MiddlewareChain,
    request::{FHIRRequest, FHIRResponse},
};

use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::claims::SubscriptionTier;

use std::sync::Arc;

pub struct Middleware {}
impl Middleware {
    pub fn new() -> Self {
        Middleware {}
    }
}

fn get_request_limit(
    subscription_tier: &SubscriptionTier,
    request: &FHIRRequest,
) -> Result<TenantResourceLimit, OperationOutcomeError> {
    match request {
        FHIRRequest::Update(_) | FHIRRequest::Create(_) => {
            let Some(resource_type) = request_to_resource_type(request) else {
                return Err(OperationOutcomeError::fatal(
                    IssueType::Exception(None),
                    "Unable to determine resource type for request".to_string(),
                ));
            };

            Ok(get_tenant_resource_limit(subscription_tier, &resource_type))
        }

        _ => Ok(TenantResourceLimit::Unlimited),
    }
}

impl<
    State: Send + Sync + Clone + 'static,
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
            let Some(next) = next else {
                return Err(OperationOutcomeError::fatal(
                    IssueType::Exception(None),
                    "No next middleware found".to_string(),
                ));
            };

            let request_limit =
                get_request_limit(&context.ctx.user.claims.subscription_tier, &context.request)?;

            match request_limit {
                TenantResourceLimit::Count(resource_type, limit) => {
                    let result = context
                        .ctx
                        .client
                        .search_type(context.ctx.clone(), resource_type.clone(), "?_total=accurate".try_into().map_err(|e|{
                            tracing::error!("Failed to construct search query for subscription tier limit middleware: {}", e);

                            OperationOutcomeError::fatal(
                                IssueType::Exception(None),
                                "Failed to construct search query for subscription tier limit middleware".to_string(),
                            )
                        })?)
                        .await?;

                    let total = result.total.and_then(|total| total.value).ok_or_else(|| {
                        OperationOutcomeError::fatal(
                            IssueType::Exception(None),
                            "Failed to retrieve total count for resource type".to_string(),
                        )
                    })?;

                    if total >= (limit as u64) {
                        return Err(OperationOutcomeError::error(
                            IssueType::TooCostly(None),
                            format!(
                                "Request exceeds the limit of '{}' for resource type '{}' for subscription tier {:?}",
                                limit,
                                resource_type.as_ref(),
                                context.ctx.user.claims.subscription_tier
                            ),
                        ));
                    }

                    next(state, context).await
                }
                TenantResourceLimit::Unlimited => return next(state, context).await,
            }
        })
    }
}
