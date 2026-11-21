use crate::fhir_client::{
    ClientState, ServerCTX,
    middleware::{Backend, Middleware, Next, Output},
};
use haste_fhir_client::request::{
    FHIRRequest, FHIRResponse, FHIRSearchTypeResponse, SearchRequest, SearchResponse,
};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId};
use std::sync::Arc;

/// Artifacts (StructureDefinitions, ValueSets, ...) are read from the system
/// tenant beside the caller's project: a read falls back to it and a search
/// lists both.
pub struct SetArtifactTenant;

fn system_artifact_tenant<B: Backend>(ctx: &ServerCTX<B>) -> Arc<ServerCTX<B>> {
    Arc::new(ServerCTX::new(
        TenantId::System,
        ProjectId::System,
        ctx.fhir_version.clone(),
        ctx.user.clone(),
        ctx.client.clone(),
        ctx.rate_limit.clone(),
    ))
}

/// One search bundle from the project's and the system tenant's: the entries
/// of both, with the totals added.
fn zip_together_search_requests(
    project_response: FHIRResponse,
    system_response: FHIRResponse,
) -> Result<FHIRResponse, OperationOutcomeError> {
    match (project_response, system_response) {
        (
            FHIRResponse::Search(SearchResponse::Type(project_search_response)),
            FHIRResponse::Search(SearchResponse::Type(system_search_response)),
        ) => {
            let mut combined_bundle = project_search_response.bundle;

            let mut combined_entries = combined_bundle.entry.unwrap_or_default();
            combined_entries.extend(system_search_response.bundle.entry.unwrap_or_default());

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
            IssueType::exception(),
            "Unexpected request type".to_string(),
        )),
    }
}

impl<B: Backend> Middleware<B> for SetArtifactTenant {
    fn call<'a>(
        &'a self,
        state: &'a ClientState<B>,
        ctx: Arc<ServerCTX<B>>,
        request: FHIRRequest,
        next: Next<'a, B>,
    ) -> Output<'a> {
        Box::pin(async move {
            match &request {
                // Writes stay in the caller's tenant and project, so they can
                // never modify the shared system artifacts. A patch reads the
                // resource from the same place, so a system artifact isn't
                // found rather than patched.
                FHIRRequest::Delete(_)
                | FHIRRequest::Update(_)
                | FHIRRequest::Patch(_)
                | FHIRRequest::Create(_) => next.run(state, ctx, request).await,
                // For search and reads look at both the system tenant that contains core resources
                // and the users tenant and current project.
                FHIRRequest::Read(_) => {
                    let response = next.run(state, ctx.clone(), request.clone()).await?;
                    if let FHIRResponse::Read(read_response) = &response
                        && read_response.resource.is_some()
                    {
                        return Ok(response);
                    }
                    next.run(state, system_artifact_tenant(&ctx), request)
                        .await
                }
                FHIRRequest::Search(SearchRequest::Type(_)) => {
                    let project_response = next.run(state, ctx.clone(), request.clone()).await?;
                    let system_response = next
                        .run(state, system_artifact_tenant(&ctx), request)
                        .await?;
                    zip_together_search_requests(project_response, system_response)
                }
                _ => Err(OperationOutcomeError::fatal(
                    IssueType::exception(),
                    "Artifact tenant middleware only supports create, read, update, patch, delete and search requests."
                        .to_string(),
                )),
            }
        })
    }
}
