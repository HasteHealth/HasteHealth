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
    request::{FHIRRequest, FHIRResponse},
};
use oxidized_fhir_model::r4::generated::{
    resources::{Resource, ResourceType, User},
    terminology::{IssueType, SupportedFhirVersion},
};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_repository::{
    Repository,
    admin::TenantAuthAdmin,
    types::{
        ProjectId, SupportedFHIRVersions,
        project::CreateProject,
        user::{AuthMethod, CreateUser, UpdateUser},
    },
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
            if let Some(resource_type) = request_to_resource_type(&context.request)
                && *resource_type != ResourceType::Project
            {
                Err(OperationOutcomeError::fatal(
                    IssueType::NotSupported(None),
                    "Only Project resource operations are supported.".to_string(),
                ))?
            } else if let Some(next) = next {
                match &context.request {
                    FHIRRequest::Create(create_request) => {
                        if let Resource::Project(project) = &create_request.resource
                            && let Some(id) = project.id.as_ref()
                        {
                            let fhir_version = match &*project.fhirVersion {
                                SupportedFhirVersion::R4(_) => Ok(SupportedFHIRVersions::R4),
                                _ => Err(OperationOutcomeError::fatal(
                                    IssueType::Invalid(None),
                                    "Invalid FHIR Version".to_string(),
                                )),
                            }?;

                            TenantAuthAdmin::create(
                                state.repo.as_ref(),
                                &context.ctx.tenant,
                                CreateProject {
                                    id: Some(ProjectId::new(id.to_string())),
                                    tenant: context.ctx.tenant.clone(),
                                    fhir_version,
                                },
                            )
                            .await?;

                            Ok(())
                        } else {
                            Err(OperationOutcomeError::fatal(
                                IssueType::Invalid(None),
                                "Project resource is invalid.".to_string(),
                            ))
                        }
                    }

                    FHIRRequest::DeleteInstance(delete_request) => {
                        if let ResourceType::Project = &delete_request.resource_type {
                            TenantAuthAdmin::<CreateProject, _, _, _>::delete(
                                state.repo.as_ref(),
                                &context.ctx.tenant,
                                &delete_request.id,
                            )
                            .await?;

                            Ok(())
                        } else {
                            Err(OperationOutcomeError::fatal(
                                IssueType::Invalid(None),
                                "Project resource is invalid.".to_string(),
                            ))
                        }
                    }

                    FHIRRequest::ConditionalUpdate(req) => {
                        if req.resource_type == ResourceType::Project {
                            Err(OperationOutcomeError::fatal(
                                IssueType::NotSupported(None),
                                "Project updates are not supported.".to_string(),
                            ))
                        } else {
                            Ok(())
                        }
                    }
                    FHIRRequest::UpdateInstance(req) => {
                        if req.resource_type == ResourceType::Project {
                            Err(OperationOutcomeError::fatal(
                                IssueType::NotSupported(None),
                                "Project updates are not supported.".to_string(),
                            ))
                        } else {
                            Ok(())
                        }
                    }

                    _ => Ok(()),
                }?;

                let res = next(state.clone(), context).await?;

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
