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

fn get_user_id<'a>(membership: &'a Membership) -> Option<&'a str> {
    if let Some(user_reference) = membership
        .user
        .reference
        .as_ref()
        .and_then(|r| r.value.as_ref())
        && let Some(user_id) = user_reference.split('/').last()
    {
        Some(user_id)
    } else {
        None
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

                    match res.response.as_ref() {
                        Some(FHIRResponse::Create(create_response)) => {
                            if let Resource::Membership(membership) = &create_response.resource
                                && let Some(user_id) = get_user_id(membership)
                            {
                                ProjectAuthAdmin::create(
                                    repo_client.as_ref(),
                                    &res.ctx.tenant,
                                    &res.ctx.project,
                                    m::CreateMembership {
                                        role: m::MembershipRole::Member,
                                        user_id: user_id.to_string(),
                                    },
                                )
                                .await?;

                                Ok(())
                            } else {
                                Err(OperationOutcomeError::fatal(
                                    IssueType::Invalid(None),
                                    "Membership resource must have a valid user reference."
                                        .to_string(),
                                ))
                            }
                        }
                        Some(FHIRResponse::DeleteInstance(delete_response)) => {
                            if let Resource::Membership(membership) = &delete_response.resource
                                && let Some(user_id) = get_user_id(membership)
                            {
                                ProjectAuthAdmin::<CreateMembership, _, _, _>::delete(
                                    repo_client.as_ref(),
                                    &res.ctx.tenant,
                                    &res.ctx.project,
                                    user_id,
                                )
                                .await?;

                                Ok(())
                            } else {
                                Err(OperationOutcomeError::fatal(
                                    IssueType::Invalid(None),
                                    "Membership resource must have a valid user reference."
                                        .to_string(),
                                ))
                            }
                        }
                        Some(FHIRResponse::Update(update_response)) => {
                            if let Resource::Membership(membership) = &update_response.resource
                                && let Some(user_id) = get_user_id(membership)
                            {
                                ProjectAuthAdmin::update(
                                    repo_client.as_ref(),
                                    &res.ctx.tenant,
                                    &res.ctx.project,
                                    m::UpdateMembership {
                                        role: m::MembershipRole::Member,
                                        user_id: user_id.to_string(),
                                    },
                                )
                                .await?;

                                Ok(())
                            } else {
                                Err(OperationOutcomeError::fatal(
                                    IssueType::Invalid(None),
                                    "Membership resource must have a valid user reference."
                                        .to_string(),
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
