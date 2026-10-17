use crate::fhir_client::{
    ClientState, ServerCTX,
    middleware::{Backend, Middleware, Next, Output},
    utilities::request_to_resource_type,
};
use haste_fhir_client::request::{
    DeleteRequest, FHIRRequest, FHIRUpdateInstanceRequest, HistoryRequest, SearchRequest,
    UpdateRequest,
};
use haste_fhir_model::r4::generated::{
    resources::{Project, Resource, ResourceType},
    terminology::{IssueType, SupportedFhirVersion},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{AuthorKind, ProjectId};
use haste_repository::{
    admin::TenantModelAdmin,
    types::{
        SupportedFHIRVersions,
        project::{CreateProject, Project as ProjectModel},
    },
    utilities::generate_hostname_id,
};
use std::sync::Arc;

/// The FHIR version a `Project` resource names, which the server must support.
fn fhir_version(project: &Project) -> Result<SupportedFHIRVersions, OperationOutcomeError> {
    if project.fhirVersion == SupportedFhirVersion::r4() {
        Ok(SupportedFHIRVersions::R4)
    } else {
        Err(OperationOutcomeError::fatal(
            IssueType::invalid(),
            format!("Invalid FHIR Version '{:?}'", project.fhirVersion),
        ))
    }
}

fn invalid_project() -> OperationOutcomeError {
    OperationOutcomeError::fatal(
        IssueType::invalid(),
        "Project resource is invalid.".to_string(),
    )
}

/// Keeps the `projects` table in step with `Project` resources, and refuses
/// the changes that would break what a project already stores.
pub struct ProjectSync;

impl<B: Backend> Middleware<B> for ProjectSync {
    fn call<'a>(
        &'a self,
        state: &'a ClientState<B>,
        ctx: Arc<ServerCTX<B>>,
        request: FHIRRequest,
        next: Next<'a, B>,
    ) -> Output<'a> {
        // Skip if not a project resource.
        if request_to_resource_type(&request)
            .is_some_and(|resource_type| *resource_type != ResourceType::Project)
        {
            return next.run(state, ctx, request);
        }

        Box::pin(async move {
            match &request {
                FHIRRequest::Create(create_request) => {
                    let Resource::Project(project) = &create_request.resource else {
                        return Err(invalid_project());
                    };
                    let fhir_version = fhir_version(project)?;

                    let name = project.name.clone();
                    let id = project
                        .id
                        .clone()
                        .unwrap_or_else(|| generate_hostname_id(Some(8)));

                    let project_model = TenantModelAdmin::create(
                        state.repo.as_ref(),
                        &ctx.tenant,
                        CreateProject {
                            id: Some(ProjectId::new(id.clone())),
                            tenant: ctx.tenant.clone(),
                            fhir_version,
                            system_created: ctx.user.claims.resource_type == AuthorKind::System,
                        },
                    )
                    .await?;

                    // Stored with the id the row got, as an update.
                    let update =
                        FHIRRequest::Update(UpdateRequest::Instance(FHIRUpdateInstanceRequest {
                            resource_type: ResourceType::Project,
                            id: id.clone(),
                            resource: Resource::Project(Project {
                                id: Some(id),
                                name,
                                fhirVersion: match project_model.fhir_version {
                                    SupportedFHIRVersions::R4 => SupportedFhirVersion::r4(),
                                },
                                ..Default::default()
                            }),
                        }));
                    next.run(state, ctx, update).await
                }

                FHIRRequest::Update(UpdateRequest::Instance(update_request)) => {
                    let Resource::Project(project) = &update_request.resource else {
                        return Err(invalid_project());
                    };
                    let fhir_version = fhir_version(project)?;

                    let Some(cur_model) = TenantModelAdmin::<CreateProject, _, _, _, _>::read(
                        state.repo.as_ref(),
                        &ctx.tenant,
                        &update_request.id,
                    )
                    .await?
                    else {
                        return Err(OperationOutcomeError::fatal(
                            IssueType::not_found(),
                            "Project not found.".to_string(),
                        ));
                    };

                    if cur_model.fhir_version != fhir_version {
                        return Err(OperationOutcomeError::fatal(
                            IssueType::not_supported(),
                            "Changing FHIR version of existing project is not supported."
                                .to_string(),
                        ));
                    }

                    if cur_model.system_created {
                        return Err(OperationOutcomeError::fatal(
                            IssueType::not_supported(),
                            "Cannot update system created projects.".to_string(),
                        ));
                    }

                    TenantModelAdmin::update(
                        state.repo.as_ref(),
                        &ctx.tenant,
                        ProjectModel {
                            id: ProjectId::new(update_request.id.clone()),
                            tenant: ctx.tenant.clone(),
                            fhir_version: cur_model.fhir_version,
                            system_created: false,
                        },
                    )
                    .await?;

                    next.run(state, ctx, request).await
                }

                FHIRRequest::Delete(DeleteRequest::Instance(delete_request)) => {
                    TenantModelAdmin::<CreateProject, _, _, _, _>::delete(
                        state.repo.as_ref(),
                        &ctx.tenant,
                        &delete_request.id,
                    )
                    .await?;

                    next.run(state, ctx, request).await
                }

                FHIRRequest::Read(_)
                | FHIRRequest::VersionRead(_)
                | FHIRRequest::Search(SearchRequest::Type(_))
                | FHIRRequest::History(HistoryRequest::Instance(_) | HistoryRequest::Type(_)) => {
                    next.run(state, ctx, request).await
                }

                // Projects are deleted one at a time, by id.
                FHIRRequest::Delete(DeleteRequest::Type(_)) => Err(OperationOutcomeError::fatal(
                    IssueType::not_supported(),
                    "Projects cannot be deleted by search. Delete each one by id.".to_string(),
                )),

                // Dissallow updates on project because could impact integrity of system. For example project has stored
                // resources in a specific FHIR version, changing that version would cause issues.
                _ => Err(OperationOutcomeError::fatal(
                    IssueType::not_supported(),
                    "Operation is not supported for Project resource types.".to_string(),
                )),
            }
        })
    }
}
