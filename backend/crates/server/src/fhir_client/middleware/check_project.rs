#![allow(unused)]
use crate::fhir_client::{
    ServerCTX,
    middleware::{
        ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput,
        ServerMiddlewareState,
    },
};
use haste_fhir_client::{
    middleware::MiddlewareChain,
    request::{FHIRRequest, FHIRResponse},
};

use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::ProjectId;
use haste_repository::Repository;
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

pub struct SetProjectReadOnlyMiddleware {
    project_id: ProjectId,
}
impl SetProjectReadOnlyMiddleware {
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
    > for SetProjectReadOnlyMiddleware
{
    fn call(
        &self,
        state: ServerMiddlewareState<Repo, Search, Terminology>,
        mut context: ServerMiddlewareContext<Repo, Search, Terminology>,
        next: Option<Arc<ServerMiddlewareNext<Repo, Search, Terminology>>>,
    ) -> ServerMiddlewareOutput<Repo, Search, Terminology> {
        let project_id = self.project_id.clone();
        Box::pin(async move {
            if let Some(next) = next {
                match &context.request {
                    FHIRRequest::Read(_) | FHIRRequest::VersionRead(_) | FHIRRequest::Search(_) => {
                        context.ctx = Arc::new(ServerCTX::new(
                            context.ctx.tenant.clone(),
                            project_id,
                            context.ctx.fhir_version.clone(),
                            context.ctx.user.clone(),
                            context.ctx.client.clone(),
                        ));
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
