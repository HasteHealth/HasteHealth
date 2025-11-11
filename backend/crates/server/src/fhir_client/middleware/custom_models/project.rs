use crate::fhir_client::{
    ClientState, ServerCTX,
    middleware::{
        ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput,
        ServerMiddlewareState,
    },
    utilities::request_to_resource_type,
};
use oxidized_fhir_client::{
    middleware::MiddlewareChain,
    request::{
        FHIRDeleteInstanceRequest, FHIRDeleteInstanceResponse, FHIRRequest, FHIRResponse,
        FHIRUpdateInstanceRequest,
    },
};
use oxidized_fhir_model::r4::generated::{
    resources::{Project, Resource, ResourceType, User},
    terminology::{IssueType, SupportedFhirVersion},
};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_jwt::{AuthorKind, ProjectId};
use oxidized_repository::{
    Repository,
    admin::TenantAuthAdmin,
    types::{
        SupportedFHIRVersions,
        project::{CreateProject, Project as ProjectModel},
        user::{AuthMethod, CreateUser, UpdateUser},
    },
    utilities::generate_id,
};
use std::sync::Arc;

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
        mut context: ServerMiddlewareContext<Repo, Search, Terminology>,
        next: Option<Arc<ServerMiddlewareNext<Repo, Search, Terminology>>>,
    ) -> ServerMiddlewareOutput<Repo, Search, Terminology> {
        Box::pin(async move {
            if let Some(next) = next {
                // Skip if not a project resource.
                if let Some(resource_type) = request_to_resource_type(&context.request)
                    && *resource_type != ResourceType::Project
                {
                    Ok(next(state, context).await?)
                } else {
                    match &context.request {
                        FHIRRequest::Create(create_request) => {
                            if let Resource::Project(project) = &create_request.resource {
                                let fhir_version = match &*project.fhirVersion {
                                    SupportedFhirVersion::R4(_) => Ok(SupportedFHIRVersions::R4),
                                    _ => Err(OperationOutcomeError::fatal(
                                        IssueType::Invalid(None),
                                        format!(
                                            "Invalid FHIR Version '{:?}'",
                                            &*project.fhirVersion
                                        ),
                                    )),
                                }?;

                                let name = project.name.clone();
                                let id = project.id.clone().unwrap_or(generate_id(Some(8)));

                                let project_model = TenantAuthAdmin::create(
                                    state.repo.as_ref(),
                                    &context.ctx.tenant,
                                    CreateProject {
                                        id: Some(ProjectId::new(id.clone())),
                                        tenant: context.ctx.tenant.clone(),
                                        fhir_version,
                                        system_created: context.ctx.user.resource_type
                                            == AuthorKind::System,
                                    },
                                )
                                .await?;

                                let res = next(
                                    state.clone(),
                                    ServerMiddlewareContext {
                                        ctx: context.ctx.clone(),
                                        response: None,
                                        request: FHIRRequest::UpdateInstance(
                                            FHIRUpdateInstanceRequest {
                                                resource_type: ResourceType::Project,
                                                id: id.clone(),
                                                resource: Resource::Project(Project {
                                                    id: Some(id),
                                                    name: name,
                                                    fhirVersion: match project_model.fhir_version {
                                                        SupportedFHIRVersions::R4 => {
                                                            Box::new(SupportedFhirVersion::R4(None))
                                                        }
                                                        _ => unreachable!(),
                                                    },
                                                    ..Default::default()
                                                }),
                                            },
                                        ),
                                    },
                                )
                                .await?;

                                Ok(res)
                            } else {
                                Err(OperationOutcomeError::fatal(
                                    IssueType::Invalid(None),
                                    "Project resource is invalid.".to_string(),
                                ))
                            }
                        }

                        FHIRRequest::UpdateInstance(update_request) => {
                            if let Resource::Project(project) = &update_request.resource {
                                let fhir_version = match &*project.fhirVersion {
                                    SupportedFhirVersion::R4(_) => Ok(SupportedFHIRVersions::R4),
                                    _ => Err(OperationOutcomeError::fatal(
                                        IssueType::Invalid(None),
                                        format!(
                                            "Invalid FHIR Version '{:?}'",
                                            &*project.fhirVersion
                                        ),
                                    )),
                                }?;

                                let name = project.name.clone();
                                let id = update_request.id.clone();

                                let Some(cur_model) =
                                    TenantAuthAdmin::<CreateProject, _, _, _, _>::read(
                                        state.repo.as_ref(),
                                        &context.ctx.tenant,
                                        &update_request.id,
                                    )
                                    .await?
                                else {
                                    return Err(OperationOutcomeError::fatal(
                                        IssueType::NotFound(None),
                                        "Project not found.".to_string(),
                                    ));
                                };

                                if &cur_model.fhir_version != &fhir_version {
                                    return Err(OperationOutcomeError::fatal(
                                        IssueType::NotSupported(None),
                                        "Changing FHIR version of existing project is not supported."
                                            .to_string(),
                                    ));
                                }

                                if cur_model.system_created {
                                    return Err(OperationOutcomeError::fatal(
                                        IssueType::NotSupported(None),
                                        "Cannot update system created projects.".to_string(),
                                    ));
                                }

                                let project_model = TenantAuthAdmin::update(
                                    state.repo.as_ref(),
                                    &context.ctx.tenant,
                                    ProjectModel {
                                        id: ProjectId::new(id.clone()),
                                        tenant: context.ctx.tenant.clone(),
                                        fhir_version: cur_model.fhir_version,
                                        system_created: false,
                                    },
                                )
                                .await?;

                                let res = next(
                                    state.clone(),
                                    ServerMiddlewareContext {
                                        ctx: context.ctx.clone(),
                                        response: None,
                                        request: context.request,
                                    },
                                )
                                .await?;

                                Ok(res)
                            } else {
                                Err(OperationOutcomeError::fatal(
                                    IssueType::Invalid(None),
                                    "Project resource is invalid.".to_string(),
                                ))
                            }
                        }

                        FHIRRequest::DeleteInstance(delete_request) => {
                            TenantAuthAdmin::<CreateProject, _, _, _, _>::delete(
                                state.repo.as_ref(),
                                &context.ctx.tenant,
                                &delete_request.id,
                            )
                            .await?;

                            let res = next(
                                state.clone(),
                                ServerMiddlewareContext {
                                    ctx: context.ctx.clone(),
                                    response: None,
                                    request: FHIRRequest::DeleteInstance(
                                        FHIRDeleteInstanceRequest {
                                            resource_type: ResourceType::Project,
                                            id: delete_request.id.clone(),
                                        },
                                    ),
                                },
                            )
                            .await?;

                            Ok(res)
                        }

                        FHIRRequest::SearchType(_) => next(state.clone(), context).await,

                        FHIRRequest::Read(read_request) => next(state.clone(), context).await,

                        // Dissallow updates on project because could impact integrity of system. For example project has stored
                        // resources in a specific FHIR version, changing that version would cause issues.
                        _ => Err(OperationOutcomeError::fatal(
                            IssueType::NotSupported(None),
                            "Operation is not supported for Project resource types.".to_string(),
                        )),
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
