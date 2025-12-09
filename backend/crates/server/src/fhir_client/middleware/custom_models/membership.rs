use crate::fhir_client::{
    ServerCTX,
    middleware::{
        ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput,
        ServerMiddlewareState,
    },
    utilities::request_to_resource_type,
};
use haste_fhir_client::{
    middleware::MiddlewareChain,
    request::{DeleteResponse, FHIRRequest, FHIRResponse},
};
use haste_fhir_model::r4::generated::{
    resources::{Membership, Resource, ResourceType},
    terminology::IssueType,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::{
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
                if let Some(resource_type) = request_to_resource_type(&context.request)
                    && *resource_type != ResourceType::Membership
                {
                    Ok(next(state, context).await?)
                } else {
                    let res = next(state.clone(), context).await?;

                    match res.response.as_ref() {
                        Some(FHIRResponse::Create(create_response)) => {
                            if let Resource::Membership(membership) = &create_response.resource
                                && let Some(user_id) = get_user_id(membership)
                                && let Some(membership_id) = membership.id.as_ref()
                            {
                                ProjectAuthAdmin::create(
                                    state.repo.as_ref(),
                                    &res.ctx.tenant,
                                    &res.ctx.project,
                                    m::CreateMembership {
                                        resource_id: membership_id.clone(),
                                        role: m::MembershipRole::Member,
                                        user_id: user_id.to_string(),
                                    },
                                )
                                .await?;

                                Ok(res)
                            } else {
                                Err(OperationOutcomeError::fatal(
                                    IssueType::Invalid(None),
                                    "Membership resource must have a valid user reference."
                                        .to_string(),
                                ))
                            }
                        }
                        Some(FHIRResponse::Delete(DeleteResponse::Instance(delete_response))) => {
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
                                    "Membership resource must have a valid user reference."
                                        .to_string(),
                                ))
                            }
                        }
                        Some(FHIRResponse::Update(update_response)) => {
                            if let Resource::Membership(membership) = &update_response.resource
                                && let Some(user_id) = get_user_id(membership)
                                && let Some(membership_id) = membership.id.as_ref()
                            {
                                ProjectAuthAdmin::update(
                                    state.repo.as_ref(),
                                    &res.ctx.tenant,
                                    &res.ctx.project,
                                    m::Membership {
                                        tenant: res.ctx.tenant.clone(),
                                        project: res.ctx.project.clone(),
                                        resource_id: membership_id.clone(),
                                        role: m::MembershipRole::Member,
                                        user_id: user_id.to_string(),
                                    },
                                )
                                .await?;

                                Ok(res)
                            } else {
                                Err(OperationOutcomeError::fatal(
                                    IssueType::Invalid(None),
                                    "Membership resource must have a valid user reference."
                                        .to_string(),
                                ))
                            }
                        }
                        _ => Ok(res),
                    }
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
