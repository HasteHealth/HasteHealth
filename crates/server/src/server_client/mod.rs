use std::{pin::Pin, sync::Arc};

use crate::{
    SupportedFHIRVersions,
    repository::{FHIRMethod, FHIRRepository, InsertResourceRow, ProjectId, ResourceId, TenantId},
};
use fhir_client::{
    FHIRClient, ParsedParameter,
    middleware::{Context, Middleware, Next},
    request::{
        FHIRCreateRequest, FHIRCreateResponse, FHIRReadRequest, FHIRReadResponse, FHIRRequest,
        FHIRResponse,
    },
};
use fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};

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
}

type ServerMiddlewareState<Repository> = Arc<Repository>;
type ServerMiddlewareContext = Context<ServerCTX, FHIRRequest, FHIRResponse>;
type ServerMiddlewareNext<Repo> =
    Option<Arc<Next<Arc<Repo>, ServerMiddlewareContext, OperationOutcomeError>>>;
type ServerMiddlewareOutput =
    Pin<Box<dyn Future<Output = Result<ServerMiddlewareContext, OperationOutcomeError>> + Send>>;

fn storage_middleware<Repository: FHIRRepository + Send + Sync + 'static>(
    state: ServerMiddlewareState<Repository>,
    mut context: ServerMiddlewareContext,
    next: ServerMiddlewareNext<Repository>,
) -> ServerMiddlewareOutput {
    Box::pin(async move {
        let response = match &mut context.request {
            FHIRRequest::Create(create_request) => Some(FHIRResponse::Create(FHIRCreateResponse {
                resource: state
                    .insert(&mut InsertResourceRow {
                        tenant: context.ctx.tenant.to_string(),
                        project: context.ctx.project.to_string(),
                        author_id: "fake_author_id".to_string(),
                        fhir_version: context.ctx.fhir_version.clone(),
                        resource: &mut create_request.resource,
                        deleted: false,
                        request_method: "POST".to_string(),
                        author_type: "member".to_string(),
                        fhir_method: FHIRMethod::Create,
                    })
                    .await?,
            })),
            FHIRRequest::Read(read_request) => {
                let resource = state
                    .read_latest(
                        &context.ctx.tenant,
                        &context.ctx.project,
                        &ResourceId::new(read_request.id.to_string()),
                    )
                    .await?;

                Some(FHIRResponse::Read(FHIRReadResponse { resource: resource }))
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

    async fn capabilities(&self, _ctx: ServerCTX) -> fhir_model::r4::types::CapabilityStatement {
        todo!()
    }

    async fn search_system(
        &self,
        _ctx: ServerCTX,
        _parameters: Vec<ParsedParameter>,
    ) -> Result<Vec<fhir_model::r4::types::Resource>, OperationOutcomeError> {
        todo!()
    }

    async fn search_type(
        &self,
        _ctx: ServerCTX,
        _resource_type: fhir_model::r4::types::ResourceType,
        _parameters: Vec<ParsedParameter>,
    ) -> Result<Vec<fhir_model::r4::types::Resource>, OperationOutcomeError> {
        todo!()
    }

    async fn create(
        &self,
        ctx: ServerCTX,
        resource_type: fhir_model::r4::types::ResourceType,
        resource: fhir_model::r4::types::Resource,
    ) -> Result<fhir_model::r4::types::Resource, OperationOutcomeError> {
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
        _resource_type: fhir_model::r4::types::ResourceType,
        _id: String,
        _resource: fhir_model::r4::types::Resource,
    ) -> Result<fhir_model::r4::types::Resource, OperationOutcomeError> {
        todo!()
    }

    async fn conditional_update(
        &self,
        _ctx: ServerCTX,
        _resource_type: fhir_model::r4::types::ResourceType,
        _parameters: Vec<ParsedParameter>,
        _resource: fhir_model::r4::types::Resource,
    ) -> Result<fhir_model::r4::types::Resource, OperationOutcomeError> {
        todo!()
    }

    async fn patch(
        &self,
        _ctx: ServerCTX,
        _resource_type: fhir_model::r4::types::ResourceType,
        _id: String,
        _patches: json_patch::Patch,
    ) -> Result<fhir_model::r4::types::Resource, OperationOutcomeError> {
        todo!()
    }

    async fn read(
        &self,
        ctx: ServerCTX,
        resource_type: fhir_model::r4::types::ResourceType,
        id: String,
    ) -> Result<Option<fhir_model::r4::types::Resource>, OperationOutcomeError> {
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
        _resource_type: fhir_model::r4::types::ResourceType,
        _id: String,
        _version_id: String,
    ) -> Result<Option<fhir_model::r4::types::Resource>, OperationOutcomeError> {
        todo!()
    }

    async fn delete_instance(
        &self,
        _ctx: ServerCTX,
        _resource_type: fhir_model::r4::types::ResourceType,
        _id: String,
    ) -> Result<(), OperationOutcomeError> {
        todo!()
    }

    async fn delete_type(
        &self,
        _ctx: ServerCTX,
        _resource_type: fhir_model::r4::types::ResourceType,
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
    ) -> Result<Vec<fhir_model::r4::types::Resource>, OperationOutcomeError> {
        todo!()
    }

    async fn history_type(
        &self,
        _ctx: ServerCTX,
        _resource_type: fhir_model::r4::types::ResourceType,
        _parameters: Vec<ParsedParameter>,
    ) -> Result<Vec<fhir_model::r4::types::Resource>, OperationOutcomeError> {
        todo!()
    }

    async fn history_instance(
        &self,
        _ctx: ServerCTX,
        _resource_type: fhir_model::r4::types::ResourceType,
        _id: String,
        _parameters: Vec<ParsedParameter>,
    ) -> Result<Vec<fhir_model::r4::types::Resource>, OperationOutcomeError> {
        todo!()
    }

    async fn invoke_instance(
        &self,
        _ctx: ServerCTX,
        _resource_type: fhir_model::r4::types::ResourceType,
        _id: String,
        _operation: String,
        _parameters: fhir_model::r4::types::Parameters,
    ) -> Result<fhir_model::r4::types::Resource, OperationOutcomeError> {
        todo!()
    }

    async fn invoke_type(
        &self,
        _ctx: ServerCTX,
        _resource_type: fhir_model::r4::types::ResourceType,
        _operation: String,
        _parameters: fhir_model::r4::types::Parameters,
    ) -> Result<fhir_model::r4::types::Resource, OperationOutcomeError> {
        todo!()
    }

    async fn invoke_system(
        &self,
        _ctx: ServerCTX,
        _operation: String,
        _parameters: fhir_model::r4::types::Parameters,
    ) -> Result<fhir_model::r4::types::Resource, OperationOutcomeError> {
        todo!()
    }

    async fn transaction(
        &self,
        _ctx: ServerCTX,
        _bundle: fhir_model::r4::types::Resource,
    ) -> Result<fhir_model::r4::types::Resource, OperationOutcomeError> {
        todo!()
    }

    async fn batch(
        &self,
        _ctx: ServerCTX,
        _bundle: fhir_model::r4::types::Resource,
    ) -> Result<fhir_model::r4::types::Resource, OperationOutcomeError> {
        todo!()
    }
}
