use std::{any::Any, pin::Pin, sync::Arc};

use crate::{
    SupportedFHIRVersions,
    repository::{
        FHIRMethod, FHIRRepository, InsertResourceRow, ProjectId, ResourceId, TenantId, VersionId,
        utilities::{set_resource_id, set_version_id},
    },
};
use oxidized_fhir_client::{
    FHIRClient, ParsedParameter,
    middleware::{Context, Middleware, MiddlewareOutput, Next},
    request::{
        FHIRCreateRequest, FHIRCreateResponse, FHIRHistoryInstanceResponse, FHIRReadRequest,
        FHIRReadResponse, FHIRRequest, FHIRResponse, FHIRUpdateResponse, FHIRVersionReadRequest,
        FHIRVersionReadResponse,
    },
};
use oxidized_fhir_model::r4::types::Resource;
use oxidized_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};

pub struct ServerCTX {
    pub tenant: TenantId,
    pub project: ProjectId,
    pub fhir_version: SupportedFHIRVersions,
}

#[derive(OperationOutcomeError, Debug)]
pub enum StorageError {
    #[error(
        code = "not-supported",
        diagnostic = "Storage not supported for fhir method."
    )]
    NotSupported,
    #[error(code = "exception", diagnostic = "No response.")]
    NoResponse,
    #[error(code = "not-found", diagnostic = "Resource not found.")]
    NotFound,
    #[error(code = "invalid", diagnostic = "Invalid resource type.")]
    InvalidType,
}

type ServerMiddlewareState<Repository> = Arc<Repository>;
type ServerMiddlewareContext = Context<ServerCTX, FHIRRequest, FHIRResponse>;
type ServerMiddlewareNext<Repo> = Next<Arc<Repo>, ServerMiddlewareContext, OperationOutcomeError>;
type ServerMiddlewareOutput = MiddlewareOutput<ServerMiddlewareContext, OperationOutcomeError>;

async fn create_resource<Repository: FHIRRepository + Send + Sync + 'static>(
    repo: Arc<Repository>,
    context: &ServerCTX,
    resource: &mut Resource,
) -> Result<Resource, OperationOutcomeError> {
    set_resource_id(resource, None)?;
    set_version_id(resource)?;
    let result = repo
        .insert(&mut InsertResourceRow {
            tenant: context.tenant.to_string(),
            project: context.project.to_string(),
            fhir_version: context.fhir_version.clone(),
            resource: resource,
            deleted: false,
            request_method: "POST".to_string(),
            author_type: "Membership".to_string(),
            author_id: "fake_author_id".to_string(),
            fhir_method: FHIRMethod::Create,
        })
        .await;

    result
}

async fn update_resource<Repository: FHIRRepository + Send + Sync + 'static>(
    repo: Arc<Repository>,
    id: &str,
    context: &ServerCTX,
    resource: &mut Resource,
) -> Result<Resource, OperationOutcomeError> {
    set_resource_id(resource, Some(id.to_string()))?;
    set_version_id(resource)?;
    let result = repo
        .insert(&mut InsertResourceRow {
            tenant: context.tenant.to_string(),
            project: context.project.to_string(),
            fhir_version: context.fhir_version.clone(),
            resource: resource,
            deleted: false,
            request_method: "PUT".to_string(),
            author_type: "Membership".to_string(),
            author_id: "fake_author_id".to_string(),
            fhir_method: FHIRMethod::Update,
        })
        .await;

    result
}

fn storage_middleware<Repository: FHIRRepository + Send + Sync + 'static>(
    state: ServerMiddlewareState<Repository>,
    mut context: ServerMiddlewareContext,
    next: Option<Arc<ServerMiddlewareNext<Repository>>>,
) -> ServerMiddlewareOutput {
    Box::pin(async move {
        let response = match &mut context.request {
            FHIRRequest::Create(create_request) => Some(FHIRResponse::Create(FHIRCreateResponse {
                resource: create_resource(
                    state.clone(),
                    &context.ctx,
                    &mut create_request.resource,
                )
                .await?,
            })),
            FHIRRequest::Read(read_request) => {
                let resource = state
                    .read_latest(
                        &context.ctx.tenant,
                        &context.ctx.project,
                        &read_request.resource_type,
                        &ResourceId::new(read_request.id.to_string()),
                    )
                    .await?
                    .ok_or_else(|| StorageError::NotFound)?;

                Some(FHIRResponse::Read(FHIRReadResponse { resource: resource }))
            }
            FHIRRequest::VersionRead(vread_request) => {
                let mut vread_resources = state
                    .read_by_version_ids(
                        &context.ctx.tenant,
                        &context.ctx.project,
                        vec![VersionId::new(&vread_request.version_id)],
                    )
                    .await?;

                if vread_resources.get(0).is_some() {
                    Some(
                        (FHIRResponse::VersionRead(FHIRVersionReadResponse {
                            resource: vread_resources.swap_remove(0),
                        })),
                    )
                } else {
                    None
                }
            }
            FHIRRequest::HistoryInstance(history_instance_request) => {
                let history_resources = state
                    .history(
                        &context.ctx.tenant,
                        &context.ctx.project,
                        &history_instance_request.resource_type,
                        &ResourceId::new(history_instance_request.id.to_string()),
                    )
                    .await?;

                Some(FHIRResponse::HistoryInstance(FHIRHistoryInstanceResponse {
                    resources: history_resources,
                }))
            }
            FHIRRequest::UpdateInstance(update_request) => {
                let resource = state
                    .read_latest(
                        &context.ctx.tenant,
                        &context.ctx.project,
                        &update_request.resource_type,
                        &ResourceId::new(update_request.id.to_string()),
                    )
                    .await?;

                if let Some(resource) = resource {
                    if std::mem::discriminant(&resource)
                        != std::mem::discriminant(&update_request.resource)
                    {
                        return Err(StorageError::InvalidType.into());
                    }

                    Some(FHIRResponse::Update(FHIRUpdateResponse {
                        resource: update_resource(
                            state.clone(),
                            &update_request.id,
                            &context.ctx,
                            &mut update_request.resource,
                        )
                        .await?,
                    }))
                } else {
                    Some(FHIRResponse::Create(FHIRCreateResponse {
                        resource: create_resource(
                            state.clone(),
                            &context.ctx,
                            &mut update_request.resource,
                        )
                        .await?,
                    }))
                }
            }
            _ => None,
        };

        let mut next_context = context;
        next_context.response = response;
        Ok(next_context)
    })
}

pub struct FHIRServerClient<Repository: FHIRRepository + Send + Sync> {
    state: Arc<Repository>,
    middleware:
        Middleware<Arc<Repository>, ServerCTX, FHIRRequest, FHIRResponse, OperationOutcomeError>,
}

impl<Repository: FHIRRepository + Send + Sync + 'static> FHIRServerClient<Repository> {
    pub fn new(repository: Repository) -> Self {
        let middleware = Middleware::new(vec![Box::new(storage_middleware)]);
        FHIRServerClient {
            state: Arc::new(repository),
            middleware,
        }
    }
}

impl<Repository: FHIRRepository + Send + Sync + 'static>
    FHIRClient<ServerCTX, OperationOutcomeError> for FHIRServerClient<Repository>
{
    async fn request(
        &self,
        _ctx: ServerCTX,
        request: FHIRRequest,
    ) -> Result<FHIRResponse, OperationOutcomeError> {
        let response = self
            .middleware
            .call(self.state.clone(), _ctx, request)
            .await?;

        response
            .response
            .ok_or_else(|| StorageError::NoResponse.into())
    }

    async fn capabilities(
        &self,
        _ctx: ServerCTX,
    ) -> oxidized_fhir_model::r4::types::CapabilityStatement {
        todo!()
    }

    async fn search_system(
        &self,
        _ctx: ServerCTX,
        _parameters: Vec<ParsedParameter>,
    ) -> Result<Vec<oxidized_fhir_model::r4::types::Resource>, OperationOutcomeError> {
        todo!()
    }

    async fn search_type(
        &self,
        _ctx: ServerCTX,
        _resource_type: oxidized_fhir_model::r4::types::ResourceType,
        _parameters: Vec<ParsedParameter>,
    ) -> Result<Vec<oxidized_fhir_model::r4::types::Resource>, OperationOutcomeError> {
        todo!()
    }

    async fn create(
        &self,
        ctx: ServerCTX,
        resource_type: oxidized_fhir_model::r4::types::ResourceType,
        resource: oxidized_fhir_model::r4::types::Resource,
    ) -> Result<oxidized_fhir_model::r4::types::Resource, OperationOutcomeError> {
        let res = self
            .middleware
            .call(
                self.state.clone(),
                ctx,
                FHIRRequest::Create(FHIRCreateRequest {
                    resource_type,
                    resource,
                }),
            )
            .await?;

        match res.response {
            Some(FHIRResponse::Create(create_response)) => Ok(create_response.resource),
            _ => panic!("Unexpected response type"),
        }
    }

    async fn update(
        &self,
        _ctx: ServerCTX,
        _resource_type: oxidized_fhir_model::r4::types::ResourceType,
        _id: String,
        _resource: oxidized_fhir_model::r4::types::Resource,
    ) -> Result<oxidized_fhir_model::r4::types::Resource, OperationOutcomeError> {
        todo!()
    }

    async fn conditional_update(
        &self,
        _ctx: ServerCTX,
        _resource_type: oxidized_fhir_model::r4::types::ResourceType,
        _parameters: Vec<ParsedParameter>,
        _resource: oxidized_fhir_model::r4::types::Resource,
    ) -> Result<oxidized_fhir_model::r4::types::Resource, OperationOutcomeError> {
        todo!()
    }

    async fn patch(
        &self,
        _ctx: ServerCTX,
        _resource_type: oxidized_fhir_model::r4::types::ResourceType,
        _id: String,
        _patches: json_patch::Patch,
    ) -> Result<oxidized_fhir_model::r4::types::Resource, OperationOutcomeError> {
        todo!()
    }

    async fn read(
        &self,
        ctx: ServerCTX,
        resource_type: oxidized_fhir_model::r4::types::ResourceType,
        id: String,
    ) -> Result<Option<oxidized_fhir_model::r4::types::Resource>, OperationOutcomeError> {
        let res = self
            .middleware
            .call(
                self.state.clone(),
                ctx,
                FHIRRequest::Read(FHIRReadRequest { resource_type, id }),
            )
            .await?;

        match res.response {
            Some(FHIRResponse::Read(read_response)) => Ok(Some(read_response.resource)),
            _ => panic!("Unexpected response type"),
        }
    }

    async fn vread(
        &self,
        _ctx: ServerCTX,
        _resource_type: oxidized_fhir_model::r4::types::ResourceType,
        _id: String,
        _version_id: String,
    ) -> Result<Option<oxidized_fhir_model::r4::types::Resource>, OperationOutcomeError> {
        todo!()
    }

    async fn delete_instance(
        &self,
        _ctx: ServerCTX,
        _resource_type: oxidized_fhir_model::r4::types::ResourceType,
        _id: String,
    ) -> Result<(), OperationOutcomeError> {
        todo!()
    }

    async fn delete_type(
        &self,
        _ctx: ServerCTX,
        _resource_type: oxidized_fhir_model::r4::types::ResourceType,
        _parameters: Vec<ParsedParameter>,
    ) -> Result<(), OperationOutcomeError> {
        todo!()
    }

    async fn delete_system(
        &self,
        _ctx: ServerCTX,
        _parameters: Vec<ParsedParameter>,
    ) -> Result<(), OperationOutcomeError> {
        todo!()
    }

    async fn history_system(
        &self,
        _ctx: ServerCTX,
        _parameters: Vec<ParsedParameter>,
    ) -> Result<Vec<oxidized_fhir_model::r4::types::Resource>, OperationOutcomeError> {
        todo!()
    }

    async fn history_type(
        &self,
        _ctx: ServerCTX,
        _resource_type: oxidized_fhir_model::r4::types::ResourceType,
        _parameters: Vec<ParsedParameter>,
    ) -> Result<Vec<oxidized_fhir_model::r4::types::Resource>, OperationOutcomeError> {
        todo!()
    }

    async fn history_instance(
        &self,
        _ctx: ServerCTX,
        _resource_type: oxidized_fhir_model::r4::types::ResourceType,
        _id: String,
        _parameters: Vec<ParsedParameter>,
    ) -> Result<Vec<oxidized_fhir_model::r4::types::Resource>, OperationOutcomeError> {
        todo!()
    }

    async fn invoke_instance(
        &self,
        _ctx: ServerCTX,
        _resource_type: oxidized_fhir_model::r4::types::ResourceType,
        _id: String,
        _operation: String,
        _parameters: oxidized_fhir_model::r4::types::Parameters,
    ) -> Result<oxidized_fhir_model::r4::types::Resource, OperationOutcomeError> {
        todo!()
    }

    async fn invoke_type(
        &self,
        _ctx: ServerCTX,
        _resource_type: oxidized_fhir_model::r4::types::ResourceType,
        _operation: String,
        _parameters: oxidized_fhir_model::r4::types::Parameters,
    ) -> Result<oxidized_fhir_model::r4::types::Resource, OperationOutcomeError> {
        todo!()
    }

    async fn invoke_system(
        &self,
        _ctx: ServerCTX,
        _operation: String,
        _parameters: oxidized_fhir_model::r4::types::Parameters,
    ) -> Result<oxidized_fhir_model::r4::types::Resource, OperationOutcomeError> {
        todo!()
    }

    async fn transaction(
        &self,
        _ctx: ServerCTX,
        _bundle: oxidized_fhir_model::r4::types::Resource,
    ) -> Result<oxidized_fhir_model::r4::types::Resource, OperationOutcomeError> {
        todo!()
    }

    async fn batch(
        &self,
        _ctx: ServerCTX,
        _bundle: oxidized_fhir_model::r4::types::Resource,
    ) -> Result<oxidized_fhir_model::r4::types::Resource, OperationOutcomeError> {
        todo!()
    }
}
