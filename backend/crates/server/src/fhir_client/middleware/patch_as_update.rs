//! Runs a PATCH as an UPDATE, for routes whose middleware only understands
//! create, update and delete (Project, User, IdentityProvider, Membership and
//! OperationDefinition).
//!
//! The patch is applied here and the patched resource is sent on as an update,
//! so it goes through the same checks as a PUT: Project refuses a new
//! `fhirVersion` or an edit to a system project, User and Membership keep
//! their `users` and `memberships` rows in sync, and OperationDefinition can't
//! be patched onto a built-in operation. The response goes back to the caller
//! as a patch response.
//!
//! Place after the transaction middleware, if the route has one, so the read,
//! patch and write happen in one transaction.
use crate::fhir_client::{
    ClientState, ServerCTX,
    middleware::{Backend, Middleware, Next, Output, storage::apply_patch},
};
use haste_fhir_client::request::{
    FHIRPatchResponse, FHIRRequest, FHIRResponse, FHIRUpdateInstanceRequest, UpdateRequest,
};
use std::sync::Arc;

pub struct PatchAsUpdate;

impl<B: Backend> Middleware<B> for PatchAsUpdate {
    fn call<'a>(
        &'a self,
        state: &'a ClientState<B>,
        ctx: Arc<ServerCTX<B>>,
        request: FHIRRequest,
        next: Next<'a, B>,
    ) -> Output<'a> {
        Box::pin(async move {
            let patch = match request {
                FHIRRequest::Patch(patch) => patch,
                other => return next.run(state, ctx, other).await,
            };

            let patched =
                apply_patch(state.repo.as_ref(), &ctx.tenant, &ctx.project, &patch).await?;
            let update = FHIRRequest::Update(UpdateRequest::Instance(FHIRUpdateInstanceRequest {
                resource_type: patch.resource_type,
                id: patch.id,
                resource: patched,
            }));

            match next.run(state, ctx, update).await? {
                FHIRResponse::Update(update) => Ok(FHIRResponse::Patch(FHIRPatchResponse {
                    resource: update.resource,
                })),
                other => Ok(other),
            }
        })
    }
}
