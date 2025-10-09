use crate::fhir_client::{
    ClientState, ServerCTX,
    middleware::{
        ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput,
        ServerMiddlewareState,
    },
    utilities::setup_transaction_context,
};
use oxidized_fhir_client::{
    middleware::MiddlewareChain,
    request::{FHIRRequest, FHIRResponse},
};
use oxidized_fhir_model::r4::generated::{
    resources::{Resource, User},
    terminology::IssueType,
};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_repository::{
    Repository,
    admin::TenantAuthAdmin,
    types::user::{AuthMethod, CreateUser, UpdateUser},
};
use std::sync::Arc;

pub struct ProjectTableAlterationMiddleware {}
impl ProjectTableAlterationMiddleware {
    pub fn new() -> Self {
        ProjectTableAlterationMiddleware {}
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
    > for ProjectTableAlterationMiddleware
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

                    match res.response.as_ref() {
                        Some(FHIRResponse::Create(create_response)) => {
                            if let Resource::User(user) = &create_response.resource
                                && let Some(email) = user.email.value.as_ref()
                                && let Some(id) = user.id.as_ref()
                            {
                                todo!("Implement project creation");

                                Ok(())
                            } else {
                                Err(OperationOutcomeError::fatal(
                                    IssueType::Invalid(None),
                                    "Project resource is invalid.".to_string(),
                                ))
                            }
                        }
                        Some(FHIRResponse::DeleteInstance(delete_response)) => {
                            if let Resource::User(user) = &delete_response.resource
                                && let Some(id) = user.id.as_ref()
                            {
                                todo!("Implement project deletion");

                                Ok(())
                            } else {
                                Err(OperationOutcomeError::fatal(
                                    IssueType::Invalid(None),
                                    "Project resource is invalid.".to_string(),
                                ))
                            }
                        }
                        Some(FHIRResponse::Update(update_response)) => {
                            if let Resource::User(user) = &update_response.resource
                                && let Some(email) = user.email.value.as_ref()
                                && let Some(id) = user.id.as_ref()
                            {
                                todo!("Implement project update.");

                                Ok(())
                            } else {
                                Err(OperationOutcomeError::fatal(
                                    IssueType::Invalid(None),
                                    "Project resource is invalid.".to_string(),
                                ))
                            }
                        }

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
