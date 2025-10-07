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
use oxidized_repository::{Repository, admin::ProjectAuthAdmin, types::membership as m};
use std::sync::Arc;

async fn setup_transaction_context<
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
            let transaction_client = Arc::new(state.repo.transaction().await?);
            Ok(Arc::new(ClientState {
                repo: transaction_client.clone(),
                search: state.search.clone(),
                terminology: state.terminology.clone(),
            }))
        }
        FHIRRequest::Read(_) | FHIRRequest::SearchType(_) => Ok(state),
        _ => Err(OperationOutcomeError::fatal(
            IssueType::NotSupported(None),
            "Request type not supported for membership middleware.".to_string(),
        )),
    }
}

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
            if let Some(next) = next {
                let repo_client;
                // Place in block so transaction_state gets dropped.
                let res = {
                    let transaction_state =
                        setup_transaction_context(&context.request, state.clone()).await?;
                    // Setup so can run a commit after.
                    repo_client = transaction_state.repo.clone();

                    let res = next(transaction_state.clone(), context).await?;

                    match &context.request {
                        FHIRRequest::Create(_) => {
                            let k = ProjectAuthAdmin::create(
                                repo_client.as_ref(),
                                &context.ctx.tenant,
                                &context.ctx.project,
                                m::CreateMembership {
                                    role: m::MembershipRole::Member,
                                    user_id: "asdf".to_string(),
                                },
                            )
                            .await?;

                            Ok(())
                        }
                        FHIRRequest::DeleteInstance(_)
                        | FHIRRequest::UpdateInstance(_)
                        | FHIRRequest::ConditionalUpdate(_) => Ok(()),
                        _ => Ok(()),
                    }?;

                    res
                };

                if repo_client.in_transaction() {
                    Arc::try_unwrap(repo_client)
                        .map_err(|_e| {
                            OperationOutcomeError::fatal(
                                IssueType::Exception(None),
                                "Failed to unwrap transaction client".to_string(),
                            )
                        })?
                        .commit()
                        .await?;
                }

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
