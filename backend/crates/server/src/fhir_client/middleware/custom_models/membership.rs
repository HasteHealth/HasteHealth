use crate::fhir_client::{
    ClientState, ServerCTX,
    middleware::{Backend, Middleware, Next, Output},
    utilities::request_to_resource_type,
};
use haste_fhir_client::request::{DeleteResponse, FHIRRequest, FHIRResponse};
use haste_fhir_model::r4::generated::{
    resources::{Resource, ResourceType},
    terminology::IssueType,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_repository::{
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

/// Keeps the `memberships` table in step with `Membership` resources.
pub struct MembershipSync;

impl<B: Backend> Middleware<B> for MembershipSync {
    fn call<'a>(
        &'a self,
        state: &'a ClientState<B>,
        ctx: Arc<ServerCTX<B>>,
        request: FHIRRequest,
        next: Next<'a, B>,
    ) -> Output<'a> {
        if request_to_resource_type(&request) != Some(&ResourceType::Membership) {
            return next.run(state, ctx, request);
        }

        Box::pin(async move {
            let response = next.run(state, ctx.clone(), request).await?;
            let (tenant, project) = (&ctx.tenant, &ctx.project);

            match &response {
                FHIRResponse::Create(create_response) => {
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
                FHIRResponse::Update(update_response) => {
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
                FHIRResponse::Delete(DeleteResponse::Instance(delete_response)) => {
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
                FHIRResponse::Read(_)
                | FHIRResponse::VersionRead(_)
                | FHIRResponse::Search(_)
                | FHIRResponse::History(_)
                | FHIRResponse::Delete(DeleteResponse::Type(_) | DeleteResponse::System(_)) => {}
                _ => {
                    return Err(OperationOutcomeError::fatal(
                        IssueType::not_supported(),
                        "Membership resources do not support this interaction.".to_string(),
                    ));
                }
            }

            Ok(response)
        })
    }
}
