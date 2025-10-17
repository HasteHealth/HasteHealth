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
use oxidized_repository::{
    Repository,
    admin::TenantAuthAdmin,
    types::{
        AuthorKind, ProjectId, SupportedFHIRVersions,
        project::CreateProject,
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
        Arc<ServerCTX>,
        FHIRRequest,
        FHIRResponse,
        OperationOutcomeError,
    > for Middleware
{
    fn call(
        &self,
        state: ServerMiddlewareState<Repo, Search, Terminology>,
        mut context: ServerMiddlewareContext,
        next: Option<Arc<ServerMiddlewareNext<Repo, Search, Terminology>>>,
    ) -> ServerMiddlewareOutput {
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
                                        system_created: context.ctx.author.kind
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

                        FHIRRequest::DeleteInstance(delete_request) => {
                            TenantAuthAdmin::<CreateProject, _, _, _>::delete(
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
