use crate::fhir_client::{
    ServerCTX,
    middleware::{
        ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput,
        ServerMiddlewareState,
    },
};
use haste_access_control::context::{PolicyContext, PolicyEnvironment, UserInfo};
use haste_fhir_client::{
    FHIRClient,
    middleware::MiddlewareChain,
    request::{FHIRRequest, FHIRResponse},
};
use haste_fhir_model::r4::generated::{resources::Resource, terminology::IssueType};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::UserRole;
use haste_repository::Repository;
use std::sync::Arc;

pub struct AccessControlMiddleware {}
impl AccessControlMiddleware {
    pub fn new() -> Self {
        Self {}
    }
}
impl<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>
    MiddlewareChain<
        ServerMiddlewareState<Repo, Search, Terminology>,
        Arc<ServerCTX<Client>>,
        FHIRRequest,
        FHIRResponse,
        OperationOutcomeError,
    > for AccessControlMiddleware
{
    fn call(
        &self,
        state: ServerMiddlewareState<Repo, Search, Terminology>,
        mut context: ServerMiddlewareContext<Client>,
        next: Option<
            Arc<ServerMiddlewareNext<Client, ServerMiddlewareState<Repo, Search, Terminology>>>,
        >,
    ) -> ServerMiddlewareOutput<Client> {
        Box::pin(async move {
            match context.ctx.user.user_role {
                // Admin and Owner roles are allowed to proceed without restrictions
                UserRole::Admin | UserRole::Owner => {
                    if let Some(next) = next {
                        return Ok(next(state, context).await?);
                    } else {
                        return Ok(context);
                    }
                }
                UserRole::Member => {
                    let policies = state
                        .repo
                        .read_by_version_ids(
                            &context.ctx.tenant,
                            &context.ctx.project,
                            &context
                                .ctx
                                .user
                                .access_policy_version_ids
                                .iter()
                                .collect::<Vec<_>>(),
                            haste_repository::fhir::CachePolicy::Cache,
                        )
                        .await?
                        .into_iter()
                        .filter_map(|v| match v {
                            Resource::AccessPolicyV2(policy) => Some(Arc::new(policy)),
                            _ => None,
                        })
                        .collect();

                    // Use System context for policy evaluation.
                    let system_ctx = Arc::new(ServerCTX::system(
                        context.ctx.tenant.clone(),
                        context.ctx.project.clone(),
                        context.ctx.client.clone(),
                        context.ctx.rate_limit.clone(),
                    ));

                    let policy_context = haste_access_control::evaluate_policies(
                        PolicyContext::new(
                            context.ctx.client.clone(),
                            system_ctx,
                            PolicyEnvironment::new(
                                context.ctx.tenant.clone(),
                                context.ctx.project.clone(),
                                context.request,
                                Arc::new(UserInfo {
                                    id: context.ctx.user.user_id.as_ref().to_string(),
                                }),
                            ),
                        ),
                        &policies,
                    )
                    .await?;

                    let req =
                        Arc::try_unwrap(policy_context.environment.request).map_err(|_| {
                            OperationOutcomeError::fatal(
                                IssueType::Exception(None),
                                "Internal error during policy evaluation.".to_string(),
                            )
                        })?;

                    context.request = FHIRRequest::from(req);

                    if let Some(next) = next {
                        Ok(next(state, context).await?)
                    } else {
                        Ok(context)
                    }
                }
            }
        })
    }
}
