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
        Arc<ServerCTX>,
        FHIRRequest,
        FHIRResponse,
        OperationOutcomeError,
    > for Middleware
{
    fn call(
        &self,
        state: ServerMiddlewareState<Repo, Search, Terminology>,
        context: ServerMiddlewareContext,
        next: Option<Arc<ServerMiddlewareNext<Repo, Search, Terminology>>>,
    ) -> ServerMiddlewareOutput {
        Box::pin(async move {
            if let Some(next) = next {
                let res = next(state.clone(), context).await?;

                match res.response.as_ref() {
                    Some(FHIRResponse::Create(create_response)) => {
                        if let Resource::Membership(membership) = &create_response.resource
                            && let Some(user_id) = get_user_id(membership)
                        {
                            ProjectAuthAdmin::create(
                                state.repo.as_ref(),
                                &res.ctx.tenant,
                                &res.ctx.project,
                                m::CreateMembership {
                                    role: m::MembershipRole::Member,
                                    user_id: user_id.to_string(),
                                },
                            )
                            .await?;

                            Ok(res)
                        } else {
                            Err(OperationOutcomeError::fatal(
                                IssueType::Invalid(None),
                                "Membership resource must have a valid user reference.".to_string(),
                            ))
                        }
                    }
                    Some(FHIRResponse::DeleteInstance(delete_response)) => {
                        if let Resource::Membership(membership) = &delete_response.resource
                            && let Some(user_id) = get_user_id(membership)
                        {
                            ProjectAuthAdmin::<CreateMembership, _, _, _, _>::delete(
                                state.repo.as_ref(),
                                &res.ctx.tenant,
                                &res.ctx.project,
                                &user_id.to_string(),
                            )
                            .await?;

                            Ok(res)
                        } else {
                            Err(OperationOutcomeError::fatal(
                                IssueType::Invalid(None),
                                "Membership resource must have a valid user reference.".to_string(),
                            ))
                        }
                    }
                    Some(FHIRResponse::Update(update_response)) => {
                        if let Resource::Membership(membership) = &update_response.resource
                            && let Some(user_id) = get_user_id(membership)
                        {
                            ProjectAuthAdmin::update(
                                state.repo.as_ref(),
                                &res.ctx.tenant,
                                &res.ctx.project,
                                m::UpdateMembership {
                                    role: m::MembershipRole::Member,
                                    user_id: user_id.to_string(),
                                },
                            )
                            .await?;

                            Ok(res)
                        } else {
                            Err(OperationOutcomeError::fatal(
                                IssueType::Invalid(None),
                                "Membership resource must have a valid user reference.".to_string(),
                            ))
                        }
                    }
                    _ => Ok(res),
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
