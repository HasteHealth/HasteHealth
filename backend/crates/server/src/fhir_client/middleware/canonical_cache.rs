use crate::fhir_client::{
    ClientState, ServerCTX,
    middleware::{Backend, Middleware, Next, Output},
    resolver,
};
use haste_fhir_client::request::FHIRRequest;
use std::sync::Arc;

/// After a canonical resource (a `StructureDefinition`, `ValueSet`, `CodeSystem`
/// or `SearchParameter`) is written, forgets the canonicals the resolver found
/// missing in the project, so a profile that was named before it existed
/// validates writes as soon as it is indexed.
pub struct CanonicalCacheInvalidation;

fn is_write(request: &FHIRRequest) -> bool {
    matches!(
        request,
        FHIRRequest::Create(_)
            | FHIRRequest::Update(_)
            | FHIRRequest::Patch(_)
            | FHIRRequest::Delete(_)
    )
}

impl<B: Backend> Middleware<B> for CanonicalCacheInvalidation {
    fn call<'a>(
        &'a self,
        state: &'a ClientState<B>,
        ctx: Arc<ServerCTX<B>>,
        request: FHIRRequest,
        next: Next<'a, B>,
    ) -> Output<'a> {
        Box::pin(async move {
            let write = is_write(&request);
            let response = next.run(state, ctx.clone(), request).await;
            if write && response.is_ok() {
                resolver::invalidate_misses(&ctx.tenant, &ctx.project);
            }
            response
        })
    }
}
