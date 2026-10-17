use crate::fhir_client::{
    ServerCTX,
    middleware::{
        ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput,
        ServerMiddlewareState,
    },
    utilities::request_to_resource_type,
};
use haste_fhir_client::{
    FHIRClient,
    middleware::MiddlewareChain,
    request::{DeleteResponse, FHIRRequest, FHIRResponse},
};
use haste_fhir_model::r4::generated::{
    resources::{Resource, ResourceType},
    terminology::IssueType,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::{
    Repository,
    admin::ProjectModelAdmin,
    types::membership::{self as m, CreateMembership},
};
use std::sync::Arc;

/// The id of the `Membership` in a response and the id of the user it references.
fn membership_ids(resource: &Resource) -> Result<(&str, &str), OperationOutcomeError> {
    if let Resource::Membership(membership) = resource
        && let Some(id) = membership.id.as_deref()
        && let Some(user_id) = membership
            .user
            .reference
            .as_ref()
            .and_then(|r| r.value.as_deref())
            .and_then(|reference| reference.split('/').next_back())
    {
        Ok((id, user_id))
    } else {
        Err(OperationOutcomeError::fatal(
            IssueType::invalid(),
            "Membership resource must have a valid user reference.".to_string(),
        ))
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
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>
    MiddlewareChain<
        ServerMiddlewareState<Repo, Search, Terminology>,
        Arc<ServerCTX<Client>>,
        FHIRRequest,
        FHIRResponse,
        OperationOutcomeError,
    > for Middleware
{
    fn call(
        &self,
        state: ServerMiddlewareState<Repo, Search, Terminology>,
        context: ServerMiddlewareContext<Client>,
        next: Option<
            Arc<ServerMiddlewareNext<Client, ServerMiddlewareState<Repo, Search, Terminology>>>,
        >,
    ) -> ServerMiddlewareOutput<Client> {
        Box::pin(async move {
            let Some(next) = next else {
                return Err(OperationOutcomeError::fatal(
                    IssueType::exception(),
                    "No next middleware found".to_string(),
                ));
            };

            if request_to_resource_type(&context.request) != Some(&ResourceType::Membership) {
                return next(state, context).await;
            }

            let res = next(state.clone(), context).await?;
            let (tenant, project) = (&res.ctx.tenant, &res.ctx.project);

            match res.response.as_ref() {
                Some(FHIRResponse::Create(create_response)) => {
                    let (id, user_id) = membership_ids(&create_response.resource)?;

                    ProjectModelAdmin::create(
                        state.repo.as_ref(),
                        tenant,
                        project,
                        m::CreateMembership {
                            resource_id: id.to_string(),
                            role: m::MembershipRole::Member,
                            user_id: user_id.to_string(),
                        },
                    )
                    .await?;
                }
                Some(FHIRResponse::Update(update_response)) => {
                    let (id, user_id) = membership_ids(&update_response.resource)?;

                    ProjectModelAdmin::update(
                        state.repo.as_ref(),
                        tenant,
                        project,
                        m::Membership {
                            tenant: tenant.clone(),
                            project: project.clone(),
                            resource_id: id.to_string(),
                            role: m::MembershipRole::Member,
                            user_id: user_id.to_string(),
                        },
                    )
                    .await?;
                }
                Some(FHIRResponse::Delete(DeleteResponse::Instance(delete_response))) => {
                    let (_, user_id) = membership_ids(&delete_response.resource)?;

                    ProjectModelAdmin::<CreateMembership, _, _, _, _>::delete(
                        state.repo.as_ref(),
                        tenant,
                        project,
                        &user_id.to_string(),
                    )
                    .await?;
                }
                // Storage runs a type or system delete as instance deletes
                // through the client, each synced above.
                Some(
                    FHIRResponse::Read(_)
                    | FHIRResponse::VersionRead(_)
                    | FHIRResponse::Search(_)
                    | FHIRResponse::History(_)
                    | FHIRResponse::Delete(DeleteResponse::Type(_) | DeleteResponse::System(_)),
                ) => {}
                _ => {
                    return Err(OperationOutcomeError::fatal(
                        IssueType::not_supported(),
                        "Membership resources do not support this interaction.".to_string(),
                    ));
                }
            }

            Ok(res)
        })
    }
}
