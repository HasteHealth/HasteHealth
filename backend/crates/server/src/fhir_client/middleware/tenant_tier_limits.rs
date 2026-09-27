/// For meta resources that require compute intensive operations we want to be able to limit access based on the users subscription tier.
///  This middleware enforces those limits.
use crate::fhir_client::{
    ServerCTX,
    middleware::{ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput},
    subscription_limits::resource_limits::{TenantResourceLimit, get_tenant_resource_limits},
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
                    IssueType::exception(),
                    "No next middleware found".to_string(),
                ));
            };

            let request_limits =
                get_request_limits(&context.ctx.user.claims.subscription_tier, &context.request)?;

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
                        context
                            .ctx
                            .client
                            .search_type(context.ctx.clone(), resource_type.clone(), parameters)
                            .await?
                    }
                    None => {
                        context
                            .ctx
                            .client
                            .search_system(context.ctx.clone(), parameters)
                            .await?
                    }
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
                            limit, scope, context.ctx.user.claims.subscription_tier
                        ),
                    ));
                }
            }

            next(state, context).await
        })
    }
}
