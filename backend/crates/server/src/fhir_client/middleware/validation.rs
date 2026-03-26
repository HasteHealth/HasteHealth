use crate::fhir_client::{
    ServerCTX,
    middleware::{
        ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput,
        ServerMiddlewareState,
    },
    resolver::ServerCTXResolver,
};
use haste_fhir_client::{
    FHIRClient,
    middleware::MiddlewareChain,
    request::{FHIRRequest, FHIRResponse, UpdateRequest},
};
use haste_fhir_model::r4::generated::{resources::Resource, types::Meta};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_profiling::FHIRProfileArguments;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_reflect::MetaValue;
use haste_repository::Repository;
use std::sync::Arc;

fn extract_profile_url_from_resource<'a>(resource: &'a Resource) -> Option<Vec<&'a str>> {
    let meta = resource.get_field("meta")?;
    let meta = meta.as_any().downcast_ref::<Box<Meta>>()?;

    meta.profile
        .as_ref()?
        .iter()
        .filter_map(|p| p.value.as_ref().map(|v| v.as_str()))
        .collect::<Vec<&str>>()
        .into()
}

fn extract_resource_from_request<'a>(request: &'a FHIRRequest) -> Option<&'a Resource> {
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
            let Some(resource) = extract_resource_from_request(&request) else {
                return Ok(None);
            };
            let profiles_url = extract_profile_url_from_resource(resource).unwrap_or(vec![]);
            if profiles_url.len() > MAX_PROFILES_PER_RESOURCE {
                return Err(OperationOutcomeError::error(
                    haste_fhir_model::r4::generated::terminology::IssueType::Invalid(None),
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

#[allow(dead_code)]
pub struct Middleware {}
impl Middleware {
    #[allow(dead_code)]
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
        context: ServerMiddlewareContext<Client>,
        next: Option<
            Arc<ServerMiddlewareNext<Client, ServerMiddlewareState<Repo, Search, Terminology>>>,
        >,
    ) -> ServerMiddlewareOutput<Client> {
        Box::pin(async move {
            let Some(next) = next else {
                return Err(OperationOutcomeError::error(
                    haste_fhir_model::r4::generated::terminology::IssueType::Invalid(None),
                    "Next middleware must be provided for profile middleware.".to_string(),
                ));
            };

            let Some((profile_urls, resource)) =
                extract_profile_url_and_resource_from_request(&context.request)?
            else {
                let result = next(state, context).await;
                return result;
            };

            for profile_url in profile_urls.iter() {
                let resolver = ServerCTXResolver::new(context.ctx.clone());
                let issues = haste_fhir_profiling::validate_profile_by_url(
                    FHIRProfileArguments::new(Arc::new(resolver)),
                    profile_url,
                    resource,
                )
                .await?;

                println!("issues: {:?} for profile", issues);
            }

            next(state, context).await
        })
    }
}
