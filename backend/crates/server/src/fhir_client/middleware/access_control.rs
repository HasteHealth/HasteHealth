#![allow(unused)]
use crate::fhir_client::{
    FHIRServerClient, ServerCTX, ServerClientConfig,
    middleware::{
        ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput,
        ServerMiddlewareState, access_control,
    },
};
use oxidized_access_control::{PolicyContext, evaluate_policy};
use oxidized_fhir_client::{
    middleware::MiddlewareChain,
    request::{FHIRRequest, FHIRResponse},
};
use oxidized_fhir_model::r4::generated::{resources::AccessPolicyV2, terminology::IssueType};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_jwt::ProjectId;
use oxidized_repository::Repository;
use std::sync::Arc;

pub struct Middleware {
    project_id: ProjectId,
}
impl Middleware {
    pub fn new(project_id: ProjectId) -> Self {
        Self { project_id }
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
        mut context: ServerMiddlewareContext<Repo, Search, Terminology>,
        next: Option<Arc<ServerMiddlewareNext<Repo, Search, Terminology>>>,
    ) -> ServerMiddlewareOutput<Repo, Search, Terminology> {
        let project_id = self.project_id.clone();
        Box::pin(async move {
            if let Some(next) = next
                && context.ctx.project == project_id
            {
                next(state, context).await
            } else {
                Err(OperationOutcomeError::fatal(
                    IssueType::Security(None),
                    format!(
                        "Must be in project {} to access this resource, not {}",
                        project_id, context.ctx.project,
                    ),
                ))
            }
        })
    }
}

pub struct AccessControlMiddleware {
    project_id: ProjectId,
}
impl AccessControlMiddleware {
    pub fn new(project_id: ProjectId) -> Self {
        Self { project_id }
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
        let project_id = self.project_id.clone();
        Box::pin(async move {
            access_control::evaluate_policy(
                &PolicyContext {
                    client: context.ctx.client.as_ref(),
                    client_context: context.ctx.clone(),
                    environment: None,
                },
                &AccessPolicyV2 {
                    ..Default::default()
                },
            );
            todo!();
        })
    }
}
