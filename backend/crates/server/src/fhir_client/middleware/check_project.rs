use crate::fhir_client::{
    ClientState, ServerCTX,
    middleware::{Backend, Middleware, Next, Output},
};
use haste_fhir_client::request::FHIRRequest;
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::ProjectId;
use std::sync::Arc;
use tracing::instrument;

/// Refuses every request made outside `project_id`.
pub struct CheckProject {
    project_id: ProjectId,
}

impl CheckProject {
    pub fn new(project_id: ProjectId) -> Self {
        Self { project_id }
    }
}

impl<B: Backend> Middleware<B> for CheckProject {
    #[instrument(
        name = "Check Project Middleware",
        skip(self, state, ctx, request, next)
    )]
    fn call<'a>(
        &'a self,
        state: &'a ClientState<B>,
        ctx: Arc<ServerCTX<B>>,
        request: FHIRRequest,
        next: Next<'a, B>,
    ) -> Output<'a> {
        if ctx.project == self.project_id {
            next.run(state, ctx, request)
        } else {
            Box::pin(std::future::ready(Err(OperationOutcomeError::fatal(
                IssueType::security(),
                format!(
                    "Must be in project {} to access this resource, not {}",
                    self.project_id, ctx.project,
                ),
            ))))
        }
    }
}

/// Reads run in `project_id` whichever project the caller is in; writes stay
/// where they are.
pub struct SetProjectReadOnly {
    project_id: ProjectId,
}

impl SetProjectReadOnly {
    pub fn new(project_id: ProjectId) -> Self {
        Self { project_id }
    }
}

impl<B: Backend> Middleware<B> for SetProjectReadOnly {
    fn call<'a>(
        &'a self,
        state: &'a ClientState<B>,
        ctx: Arc<ServerCTX<B>>,
        request: FHIRRequest,
        next: Next<'a, B>,
    ) -> Output<'a> {
        let ctx = match &request {
            FHIRRequest::Read(_) | FHIRRequest::VersionRead(_) | FHIRRequest::Search(_) => {
                Arc::new(
                    ServerCTX::new(
                        ctx.tenant.clone(),
                        self.project_id.clone(),
                        ctx.fhir_version.clone(),
                        ctx.user.clone(),
                        ctx.client.clone(),
                        ctx.rate_limit.clone(),
                    )
                    .with_tracing_id(ctx.tracing_id.clone()),
                )
            }
            _ => ctx,
        };
        next.run(state, ctx, request)
    }
}
