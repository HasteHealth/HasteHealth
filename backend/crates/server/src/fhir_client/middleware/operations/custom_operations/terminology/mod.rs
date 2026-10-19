use crate::fhir_client::{ServerCTX, middleware::Backend};
use haste_fhir_client::request::InvocationRequest;
use haste_fhir_model::r4::generated::{
    resources::{Resource, ResourceType, ValueSet},
    terminology::IssueType,
};
use haste_fhir_operation_error::OperationOutcomeError;
use std::sync::Arc;

pub use validate_code::*;
pub use valueset_expand::*;

mod validate_code;
mod valueset_expand;

/// The id of the ValueSet an instance-level invocation (`ValueSet/{id}/$op`)
/// targets. Type and system level invocations name the ValueSet in their
/// parameters instead.
fn instance_id(request: &InvocationRequest) -> Option<String> {
    match request {
        InvocationRequest::Instance(instance) => Some(instance.id.clone()),
        InvocationRequest::Type(_) | InvocationRequest::System(_) => None,
    }
}

/// Read the instance's ValueSet. A read falls back to the system tenant, so
/// shared artifacts such as `administrative-gender` resolve here as they do
/// in a search.
async fn read_value_set<B: Backend>(
    ctx: Arc<ServerCTX<B>>,
    id: String,
) -> Result<ValueSet, OperationOutcomeError> {
    match ctx
        .client
        .read(ctx.clone(), ResourceType::ValueSet, id.clone())
        .await?
    {
        Some(Resource::ValueSet(value_set)) => Ok(value_set),
        _ => Err(OperationOutcomeError::error(
            IssueType::not_found(),
            format!("ValueSet '{id}' not found."),
        )),
    }
}
