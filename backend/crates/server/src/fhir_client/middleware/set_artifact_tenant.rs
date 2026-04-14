use crate::fhir_client::{
    ServerCTX,
    middleware::{ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput},
};
use haste_fhir_client::{
    FHIRClient,
    middleware::MiddlewareChain,
    request::{FHIRRequest, FHIRResponse, FHIRSearchTypeResponse, SearchRequest, SearchResponse},
};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId};
use std::sync::Arc;

pub struct Middleware {}
impl Middleware {
    pub fn new() -> Self {
        Middleware {}
    }
}

fn system_artifact_tenant<
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>(
    ctx: Arc<ServerCTX<Client>>,
) -> Arc<ServerCTX<Client>> {
    Arc::new(ServerCTX::new(
        TenantId::System,
        ProjectId::System,
        ctx.fhir_version.clone(),
        ctx.user.clone(),
        ctx.client.clone(),
        ctx.rate_limit.clone(),
    ))
}

fn zip_together_search_requests(
    project_request: Option<FHIRResponse>,
    system_request: Option<FHIRResponse>,
) -> Result<FHIRResponse, OperationOutcomeError> {
    // Zips together the search bundle entries together from two requests into a single search bundle. This is used to combine the results from the users tenant and the system tenant for search requests.
    match (project_request, system_request) {
        (
            Some(FHIRResponse::Search(SearchResponse::Type(project_search_response))),
            Some(FHIRResponse::Search(SearchResponse::Type(system_search_response))),
        ) => {
            let mut combined_bundle = project_search_response.bundle;

            let mut combined_entries = combined_bundle.entry.unwrap_or(vec![]);
            combined_entries.extend(system_search_response.bundle.entry.unwrap_or(vec![]));

            combined_bundle.entry = Some(combined_entries);

            combined_bundle.total = combined_bundle.total.map(|mut total| {
                total.value = Some(
                    total.value.unwrap_or(0)
                        + system_search_response
                            .bundle
                            .total
                            .and_then(|t| t.value)
                            .unwrap_or(0),
                );
                total
            });

            Ok(FHIRResponse::Search(SearchResponse::Type(
                FHIRSearchTypeResponse {
                    bundle: combined_bundle,
                },
            )))
        }
        _ => Err(OperationOutcomeError::fatal(
            IssueType::Exception(None),
            "Unexpected request type".to_string(),
        )),
    }
}

impl<
    State: Send + Sync + Clone + 'static,
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
> MiddlewareChain<State, Arc<ServerCTX<Client>>, FHIRRequest, FHIRResponse, OperationOutcomeError>
    for Middleware
{
    fn call(
        &self,
        state: State,
        context: ServerMiddlewareContext<Client>,
        next: Option<Arc<ServerMiddlewareNext<Client, State>>>,
    ) -> ServerMiddlewareOutput<Client> {
        Box::pin(async move {
            let Some(next) = next else {
                return Err(OperationOutcomeError::fatal(
                    IssueType::Exception(None),
                    "No next middleware found".to_string(),
                ));
            };

            match context.request {
                FHIRRequest::Delete(_) | FHIRRequest::Update(_) | FHIRRequest::Create(_) => {
                    next(state, context).await
                }
                // For search and reads look at both the system tenant that contains core resources
                // and the users tenant and current project.
                FHIRRequest::Read(_) => {
                    let mut project_response = next(state.clone(), context).await?;
                    if let Some(response) = project_response.response.as_ref()
                        && let FHIRResponse::Read(read_response) = response
                        && read_response.resource.is_some()
                    {
                        Ok(project_response)
                    } else {
                        project_response.ctx = system_artifact_tenant(project_response.ctx);
                        project_response.response = None;
                        next(state, project_response).await
                    }
                }
                FHIRRequest::Search(SearchRequest::Type(_)) => {
                    let mut context = next(state.clone(), context).await?;
                    let project_response = context.response;

                    context.ctx = system_artifact_tenant(context.ctx);
                    context.response = None;

                    let mut context = next(state, context).await?;
                    let system_response = context.response;

                    context.response = Some(zip_together_search_requests(
                        project_response,
                        system_response,
                    )?);

                    Ok(context)
                }
                _ => {
                    return Err(OperationOutcomeError::fatal(
                        IssueType::Exception(None),
                        "Artifact tenant middleware only supports read and search requests."
                            .to_string(),
                    ));
                }
            }
        })
    }
}
