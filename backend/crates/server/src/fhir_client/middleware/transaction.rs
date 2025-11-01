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
use oxidized_fhir_model::r4::generated::{
    resources::{Membership, Resource},
    terminology::IssueType,
};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_repository::{
    Repository,
    admin::ProjectAuthAdmin,
    types::membership::{self as m, CreateMembership},
};
use std::sync::Arc;
use tracing::info;

// Only need a transaction in the context of Create, Update, Delete, and Conditional Update.
pub async fn setup_transaction_context<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    request: &FHIRRequest,
    state: ServerMiddlewareState<Repo, Search, Terminology>,
) -> Result<ServerMiddlewareState<Repo, Search, Terminology>, OperationOutcomeError> {
    match request {
        FHIRRequest::Create(_)
        | FHIRRequest::DeleteInstance(_)
        | FHIRRequest::UpdateInstance(_)
        | FHIRRequest::ConditionalUpdate(_) => {
            if state.repo.in_transaction() {
                return Ok(state);
            } else {
                let transaction_client = Arc::new(state.repo.transaction().await?);
                Ok(Arc::new(ClientState {
                    repo: transaction_client.clone(),
                    search: state.search.clone(),
                    terminology: state.terminology.clone(),
                }))
            }
        }
        FHIRRequest::Read(_) | FHIRRequest::SearchType(_) => Ok(state),
        _ => Err(OperationOutcomeError::fatal(
            IssueType::NotSupported(None),
            "Request type not supported for membership middleware.".to_string(),
        )),
    }
}

pub struct Middleware {}
impl Middleware {
    pub fn new() -> Self {
        Middleware {}
    }
}
impl<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>
    MiddlewareChain<
        ServerMiddlewareState<Repo, Search, Terminology>,
        Arc<ServerCTX<Repo, Search, Terminology>>,
        FHIRRequest,
        FHIRResponse,
        OperationOutcomeError,
    > for Middleware
{
    fn call(
        &self,
        state: ServerMiddlewareState<Repo, Search, Terminology>,
        context: ServerMiddlewareContext<Repo, Search, Terminology>,
        next: Option<Arc<ServerMiddlewareNext<Repo, Search, Terminology>>>,
    ) -> ServerMiddlewareOutput<Repo, Search, Terminology> {
        Box::pin(async move {
            if let Some(next) = next {
                // Skip over commit which will happen from caller site.
                if state.repo.in_transaction() {
                    Ok(next(state, context).await?)
                } else {
                    let repo_client;
                    // Place in block so transaction_state gets dropped.
                    let res = {
                        let transaction_state =
                            setup_transaction_context(&context.request, state.clone()).await?;
                        // Setup so can run a commit after.
                        repo_client = transaction_state.repo.clone();
                        let res = next(transaction_state.clone(), context).await;

                        res
                    };

                    if res.is_ok() && repo_client.in_transaction() {
                        Arc::try_unwrap(repo_client)
                            .map_err(|_e| {
                                OperationOutcomeError::fatal(
                                    IssueType::Exception(None),
                                    "Failed to unwrap transaction client".to_string(),
                                )
                            })?
                            .commit()
                            .await?;
                    } else if res.is_err() && repo_client.in_transaction() {
                        info!("Rolling back transaction due to error");
                        Arc::try_unwrap(repo_client)
                            .map_err(|_e| {
                                OperationOutcomeError::fatal(
                                    IssueType::Exception(None),
                                    "Failed to unwrap transaction client".to_string(),
                                )
                            })?
                            .rollback()
                            .await?;
                    }

                    res
                }
            } else {
                Err(OperationOutcomeError::fatal(
                    IssueType::Exception(None),
                    "No next middleware found".to_string(),
                ))
            }
        })
    }
}
