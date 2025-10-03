use crate::fhir_client::{
    ServerCTX,
    middleware::{
        ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput,
        ServerMiddlewareState,
    },
};
use oxidized_fhir_client::{
    middleware::MiddlewareChain,
    request::{FHIRRequest, FHIRResponse},
};
use oxidized_fhir_model::r4::generated::{resources::ResourceType, terminology::IssueType};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_repository::Repository;
use std::sync::Arc;

fn validate_resource_type(
    resource_types_allowed: Arc<Vec<ResourceType>>,
    resource_type: &ResourceType,
) -> Result<(), OperationOutcomeError> {
    if !resource_types_allowed.contains(resource_type) {
        return Err(OperationOutcomeError::error(
            IssueType::NotSupported(None),
            format!("Resource type '{resource_type:?}' is not allowed"),
        ));
    }
    Ok(())
}

pub struct ValidateResourceTypeMiddleware {
    resource_types_allowed: Arc<Vec<ResourceType>>,
}
impl ValidateResourceTypeMiddleware {
    pub fn new(resource_types_allowed: Vec<ResourceType>) -> Self {
        ValidateResourceTypeMiddleware {
            resource_types_allowed: Arc::new(resource_types_allowed),
        }
    }
}
impl<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>
    MiddlewareChain<
        ServerMiddlewareState<Repo, Search, Terminology>,
        Arc<ServerCTX>,
        FHIRRequest,
        FHIRResponse,
        OperationOutcomeError,
    > for ValidateResourceTypeMiddleware
{
    fn call(
        &self,
        state: ServerMiddlewareState<Repo, Search, Terminology>,
        mut context: ServerMiddlewareContext,
        next: Option<Arc<ServerMiddlewareNext<Repo, Search, Terminology>>>,
    ) -> ServerMiddlewareOutput {
        let resource_types = self.resource_types_allowed.clone();
        Box::pin(async move {
            match &context.request {
                FHIRRequest::Create(request) => {
                    validate_resource_type(resource_types, &request.resource_type)
                }
                _ => Ok(()),
            }?;

            if let Some(next) = next {
                next(state, context).await
            } else {
                Err(OperationOutcomeError::fatal(
                    IssueType::Exception(None),
                    "No next middleware found".to_string(),
                ))
            }
        })
    }
}
