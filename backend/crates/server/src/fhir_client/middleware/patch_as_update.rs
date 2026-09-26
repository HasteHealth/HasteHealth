//! Runs a PATCH as an UPDATE, for routes whose middleware only understands
//! create, update and delete (the tenant and project auth resources: Project,
//! User, IdentityProvider, Membership).
//!
//! The patch is applied here and the patched resource is sent on as an update,
//! so it goes through the same checks as a PUT: Project refuses a new
//! `fhirVersion` or an edit to a system project, and User and Membership keep
//! their `users` and `memberships` rows in sync. The response goes back to the
//! caller as a patch response.
//!
//! Place after the transaction middleware, so the read, patch and write happen
//! in one transaction.
use crate::fhir_client::{
    ServerCTX,
    middleware::{
        ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput,
        ServerMiddlewareState, storage::apply_patch,
    },
};
use haste_fhir_client::{
    FHIRClient,
    middleware::MiddlewareChain,
    request::{
        FHIRPatchResponse, FHIRRequest, FHIRResponse, FHIRUpdateInstanceRequest, UpdateRequest,
    },
};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::Repository;
use std::sync::Arc;

pub struct Middleware {}
impl Middleware {
    pub fn new() -> Self {
        Middleware {}
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
    > for Middleware
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
            let Some(next) = next else {
                return Err(OperationOutcomeError::fatal(
                    IssueType::exception(),
                    "No next middleware found".to_string(),
                ));
            };

            let FHIRRequest::Patch(patch_request) = &context.request else {
                return next(state, context).await;
            };

            let patched = apply_patch(
                state.repo.as_ref(),
                &context.ctx.tenant,
                &context.ctx.project,
                patch_request,
            )
            .await?;

            context.request =
                FHIRRequest::Update(UpdateRequest::Instance(FHIRUpdateInstanceRequest {
                    resource_type: patch_request.resource_type.clone(),
                    id: patch_request.id.clone(),
                    resource: patched,
                }));

            let mut res = next(state, context).await?;
            res.response = match res.response {
                Some(FHIRResponse::Update(update)) => {
                    Some(FHIRResponse::Patch(FHIRPatchResponse {
                        resource: update.resource,
                    }))
                }
                other => other,
            };
            Ok(res)
        })
    }
}
