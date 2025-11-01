#![allow(unused)]
use crate::fhir_client::{
    FHIRServerClient, ServerCTX, ServerClientConfig, middleware::{
        ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput,
        ServerMiddlewareState, access_control,
    }
};
use oxidized_access_control::{PolicyContext, evaluate_policy};
use oxidized_fhir_client::{
    middleware::MiddlewareChain,
    request::{FHIRRequest, FHIRResponse},
};
use oxidized_fhir_model::r4::generated::terminology::IssueType;
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
        Arc<ServerCTX>,
        FHIRRequest,
        FHIRResponse,
        OperationOutcomeError,
    > for Middleware
{
    fn call(
        &self,
        state: ServerMiddlewareState<Repo, Search, Terminology>,
        mut context: ServerMiddlewareContext,
        next: Option<Arc<ServerMiddlewareNext<Repo, Search, Terminology>>>,
    ) -> ServerMiddlewareOutput {
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
        Arc<ServerCTX>,
        FHIRRequest,
        FHIRResponse,
        OperationOutcomeError,
    > for AccessControlMiddleware
{
    fn call(
        &self,
        state: ServerMiddlewareState<Repo, Search, Terminology>,
        mut context: ServerMiddlewareContext,
        next: Option<Arc<ServerMiddlewareNext<Repo, Search, Terminology>>>,
    ) -> ServerMiddlewareOutput {
        let project_id = self.project_id.clone();
        Box::pin(async move {
            access_control::evaluate_policy(&PolicyContext{
               client: &FHIRServerClient::new(ServerClientConfig{repo: state.repo.clone(), search: state.search.clone(), terminology: state.terminology.clone()}),
            }, policy)
            if let Some(next) = next {
                match &context.request {
                    FHIRRequest::Read(_)
                    | FHIRRequest::VersionRead(_)
                    | FHIRRequest::SearchSystem(_)
                    | FHIRRequest::SearchType(_) => {
                        context.ctx = Arc::new(ServerCTX {
                            tenant: context.ctx.tenant.clone(),
                            project: project_id,
                            fhir_version: context.ctx.fhir_version.clone(),
                            author: context.ctx.author.clone(),
                        });
                        next(state, context).await
                    }
                    _ => next(state, context).await,
                }
            } else {
                Err(OperationOutcomeError::fatal(
                    IssueType::Exception(None),
                    "No next middleware found".to_string(),
                ))
            }
        })
    }
}
