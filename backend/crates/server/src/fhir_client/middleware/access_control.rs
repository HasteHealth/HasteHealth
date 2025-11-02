#![allow(unused)]
use crate::fhir_client::{
    FHIRServerClient, ServerCTX, ServerClientConfig,
    middleware::{
        ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput,
        ServerMiddlewareState,
    },
};
use oxidized_access_control::PolicyContext;
use oxidized_fhir_client::{
    middleware::MiddlewareChain,
    request::{FHIRRequest, FHIRResponse},
};
use oxidized_fhir_model::r4::generated::{
    resources::{AccessPolicyV2, Resource},
    terminology::IssueType,
};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_jwt::ProjectId;
use oxidized_repository::Repository;
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
>
    MiddlewareChain<
        ServerMiddlewareState<Repo, Search, Terminology>,
        Arc<ServerCTX<Repo, Search, Terminology>>,
        FHIRRequest,
        FHIRResponse,
        OperationOutcomeError,
    > for AccessControlMiddleware
{
    fn call(
        &self,
        state: ServerMiddlewareState<Repo, Search, Terminology>,
        mut context: ServerMiddlewareContext<Repo, Search, Terminology>,
        next: Option<Arc<ServerMiddlewareNext<Repo, Search, Terminology>>>,
    ) -> ServerMiddlewareOutput<Repo, Search, Terminology> {
        Box::pin(async move {
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
                    oxidized_repository::fhir::CachePolicy::Cache,
                )
                .await?
                .into_iter()
                .filter_map(|v| match v {
                    Resource::AccessPolicyV2(policy) => Some(policy),
                    _ => None,
                })
                .collect();

            oxidized_access_control::evaluate_policies(
                &PolicyContext {
                    client: context.ctx.client.as_ref(),
                    client_context: context.ctx.clone(),
                    environment: None,
                },
                &policies,
            )
            .await?;

            if let Some(next) = next {
                Ok(next(state, context).await?)
            } else {
                Ok(context)
            }
        })
    }
}
