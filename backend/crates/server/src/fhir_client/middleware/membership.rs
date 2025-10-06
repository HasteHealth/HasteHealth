use crate::fhir_client::{
    ClientState, ServerCTX,
    middleware::{
        ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput,
        ServerMiddlewareState,
    },
};
use oxidized_fhir_client::{
    middleware::MiddlewareChain,
    request::{FHIRRequest, FHIRResponse},
};
use oxidized_fhir_model::r4::generated::terminology::IssueType;
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_repository::Repository;
use std::sync::Arc;

pub struct MembershipTableAlterationMiddleware {}
impl MembershipTableAlterationMiddleware {
    pub fn new() -> Self {
        MembershipTableAlterationMiddleware {}
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
    > for MembershipTableAlterationMiddleware
{
    fn call(
        &self,
        state: ServerMiddlewareState<Repo, Search, Terminology>,
        context: ServerMiddlewareContext,
        next: Option<Arc<ServerMiddlewareNext<Repo, Search, Terminology>>>,
    ) -> ServerMiddlewareOutput {
        Box::pin(async move {
            let transaction_client = Arc::new(state.repo.transaction().await?);

            if let Some(next) = next {
                let mut res = context;
                {
                    let transaction_state = Arc::new(ClientState {
                        repo: transaction_client.clone(),
                        search: state.search.clone(),
                        terminology: state.terminology.clone(),
                    });
                    res = next(transaction_state, res).await?;
                };
                Arc::try_unwrap(transaction_client)
                    .map_err(|_e| {
                        OperationOutcomeError::fatal(
                            IssueType::Exception(None),
                            "Failed to unwrap transaction client".to_string(),
                        )
                    })?
                    .commit()
                    .await?;
                Ok(res)
            } else {
                Err(OperationOutcomeError::fatal(
                    IssueType::Exception(None),
                    "No next middleware found".to_string(),
                ))
            }
        })
    }
}
