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
    resources::{Resource, User},
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
        context: ServerMiddlewareContext,
        next: Option<Arc<ServerMiddlewareNext<Repo, Search, Terminology>>>,
    ) -> ServerMiddlewareOutput {
        Box::pin(async move {
            if let Some(next) = next {
                let res = next(state.clone(), context).await?;

                match res.response.as_ref() {
                    Some(FHIRResponse::Create(create_response)) => {
                        if let Resource::Project(project) = &create_response.resource
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
                                &res.ctx.tenant,
                                CreateProject {
                                    id: Some(ProjectId::new(id.to_string())),
                                    tenant: res.ctx.tenant.clone(),
                                    fhir_version,
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
                    Some(FHIRResponse::DeleteInstance(delete_response)) => {
                        if let Resource::Project(project) = &delete_response.resource
                            && let Some(id) = project.id.as_ref()
                        {
                            TenantAuthAdmin::<CreateProject, _, _, _>::delete(
                                state.repo.as_ref(),
                                &res.ctx.tenant,
                                id,
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
                    Some(FHIRResponse::Update(update_response)) => {
                        if let Resource::User(user) = &update_response.resource
                            && let Some(email) = user.email.value.as_ref()
                            && let Some(id) = user.id.as_ref()
                        {
                            Err(OperationOutcomeError::fatal(
                                IssueType::NotSupported(None),
                                "Project updates are not supported.".to_string(),
                            ))
                        } else {
                            Err(OperationOutcomeError::fatal(
                                IssueType::Invalid(None),
                                "Project resource is invalid.".to_string(),
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
