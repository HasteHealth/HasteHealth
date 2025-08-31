use crate::fhir_client::middleware::{
    ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput, ServerMiddlewareState,
};
use oxidized_fhir_client::request::{FHIRCapabilitiesResponse, FHIRRequest, FHIRResponse};
use oxidized_fhir_model::r4::types::CapabilityStatement;
use oxidized_fhir_operation_error::{OperationOutcomeCodes, OperationOutcomeError};
use oxidized_fhir_search::SearchEngine;
use oxidized_repository::Repository;
use std::sync::{Arc, LazyLock, Mutex};

static CAPABILITIES: LazyLock<Mutex<Option<CapabilityStatement>>> =
    LazyLock::new(|| Mutex::new(None));

pub fn capabilities<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
>(
    state: ServerMiddlewareState<Repo, Search>,
    mut context: ServerMiddlewareContext,
    next: Option<Arc<ServerMiddlewareNext<Repo, Search>>>,
) -> ServerMiddlewareOutput {
    Box::pin(async move {
        match context.request {
            FHIRRequest::Capabilities => {
                match CAPABILITIES.lock() {
                    Ok(mut guard) => {
                        if let Some(capabilities) = &*guard {
                            context.response =
                                Some(FHIRResponse::Capabilities(FHIRCapabilitiesResponse {
                                    capabilities: capabilities.clone(),
                                }));
                        } else {
                            let capabilities = CapabilityStatement::default();
                            *guard = Some(capabilities.clone());

                            context.response =
                                Some(FHIRResponse::Capabilities(FHIRCapabilitiesResponse {
                                    capabilities: capabilities,
                                }));
                        }
                    }
                    Err(_) => {
                        // Handle the error case
                    }
                }

                Ok(context)
            }
            _ => {
                if let Some(next) = next {
                    next(state, context).await
                } else {
                    Err(OperationOutcomeError::fatal(
                        OperationOutcomeCodes::Exception,
                        "No next middleware found".to_string(),
                    ))
                }
            }
        }
    })
}
