/// For meta resources that require compute intensive operations we want to be able to limit access based on the users subscription tier.
///  This middleware enforces those limits.
use crate::fhir_client::{
    ServerCTX,
    middleware::{ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput},
};
use haste_fhir_client::{
    FHIRClient,
    middleware::MiddlewareChain,
    request::{FHIRRequest, FHIRResponse},
};

use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;

use std::sync::Arc;

pub struct Middleware {}
impl Middleware {
    pub fn new() -> Self {
        Middleware {}
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

            next(state, context).await
        })
    }
}
