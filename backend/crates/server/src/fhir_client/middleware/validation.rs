use crate::fhir_client::{
    ClientState, ServerCTX,
    middleware::{Backend, Middleware, Next, Output},
    resolver::ServerCTXResolver,
};
use haste_fhir_client::request::{FHIRRequest, UpdateRequest};
use haste_fhir_model::r4::generated::{resources::Resource, terminology::IssueType, types::Meta};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_profiling::FHIRProfileArguments;
use haste_reflect::MetaValue;
use std::sync::Arc;

fn extract_profile_url_from_resource(resource: &Resource) -> Option<Vec<&str>> {
    let meta = resource.get_field("meta")?;
    let meta = meta.as_any().downcast_ref::<Meta>()?;

    meta.profile
        .as_ref()?
        .iter()
        .filter_map(|p| p.value.as_deref())
        .collect::<Vec<&str>>()
        .into()
}

fn extract_resource_from_request(request: &FHIRRequest) -> Option<&Resource> {
    match request {
        FHIRRequest::Create(create_request) => Some(&create_request.resource),
        FHIRRequest::Update(update_request) => match update_request {
            UpdateRequest::Instance(request) => Some(&request.resource),
            UpdateRequest::Conditional(request) => Some(&request.resource),
        },
        _ => None,
    }
}

static MAX_PROFILES_PER_RESOURCE: usize = 2;

fn extract_profile_url_and_resource_from_request(
    request: &FHIRRequest,
) -> Result<Option<(Vec<&str>, &dyn haste_reflect::MetaValue)>, OperationOutcomeError> {
    match request {
        FHIRRequest::Create(_) | FHIRRequest::Update(_) => {
            let Some(resource) = extract_resource_from_request(request) else {
                return Ok(None);
            };
            let profiles_url = extract_profile_url_from_resource(resource).unwrap_or(vec![]);
            if profiles_url.len() > MAX_PROFILES_PER_RESOURCE {
                return Err(OperationOutcomeError::error(
                    IssueType::invalid(),
                    format!(
                        "Too many profiles on resource. Maximum allowed is '{}'",
                        MAX_PROFILES_PER_RESOURCE
                    ),
                ));
            }

            Ok(Some((profiles_url, resource)))
        }
        FHIRRequest::Capabilities
        | FHIRRequest::Search(_)
        | FHIRRequest::History(_)
        | FHIRRequest::Invocation(_)
        | FHIRRequest::Batch(_)
        | FHIRRequest::Transaction(_)
        | FHIRRequest::Read(_)
        | FHIRRequest::VersionRead(_)
        | FHIRRequest::Patch(_)
        | FHIRRequest::Delete(_)
        | FHIRRequest::Compartment(_) => Ok(None),
    }
}

/// Validates a written resource against the profiles its `meta.profile`
/// claims.
pub struct Validation;

impl<B: Backend> Middleware<B> for Validation {
    fn call<'a>(
        &'a self,
        state: &'a ClientState<B>,
        ctx: Arc<ServerCTX<B>>,
        request: FHIRRequest,
        next: Next<'a, B>,
    ) -> Output<'a> {
        Box::pin(async move {
            if let Some((profile_urls, resource)) =
                extract_profile_url_and_resource_from_request(&request)?
            {
                for profile_url in profile_urls {
                    let resolver = ServerCTXResolver::new(ctx.clone());
                    let validation_operation_outcome =
                        haste_fhir_profiling::validate_profile_by_url(
                            FHIRProfileArguments::new(Arc::new(resolver)),
                            profile_url,
                            resource,
                        )
                        .await?;

                    if !validation_operation_outcome.issue.is_empty() {
                        return Err(OperationOutcomeError::unprocessable(
                            validation_operation_outcome,
                        ));
                    }
                }
            }

            next.run(state, ctx, request).await
        })
    }
}
