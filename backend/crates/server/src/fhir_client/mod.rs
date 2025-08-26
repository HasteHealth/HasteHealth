use crate::fhir_client::middleware::{
    ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput, ServerMiddlewareState,
};
use oxidized_fhir_client::{
    FHIRClient,
    middleware::{Middleware, MiddlewareChain},
    request::{
        FHIRCreateRequest, FHIRReadRequest, FHIRRequest, FHIRResponse, FHIRSearchTypeRequest,
    },
    url::ParsedParameter,
};
use oxidized_fhir_model::r4::types::ResourceType;
use oxidized_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use oxidized_fhir_search::SearchEngine;
use oxidized_repository::{
    Repository,
    types::{Author, ProjectId, SupportedFHIRVersions, TenantId},
};
use std::sync::Arc;

mod middleware;

#[derive(OperationOutcomeError, Debug)]
pub enum StorageError {
    #[error(
        code = "not-supported",
        diagnostic = "Storage not supported for fhir method."
    )]
    NotSupported,
    #[error(
        code = "exception",
        diagnostic = "No response was returned from the request."
    )]
    NoResponse,
    #[error(
        code = "not-found",
        diagnostic = "Resource '{arg0:?}' with id '{arg1}' not found."
    )]
    NotFound(ResourceType, String),
    #[error(code = "invalid", diagnostic = "Invalid resource type.")]
    InvalidType,
}

pub struct ServerCTX {
    pub tenant: TenantId,
    pub project: ProjectId,
    pub fhir_version: SupportedFHIRVersions,
    pub author: Author,
}

struct ClientState<Repo: Repository + Send + Sync, Search: SearchEngine + Send + Sync> {
    repo: Repo,
    search: Arc<Search>,
}

struct Route<Repo: Repository + Send + Sync + 'static, Search: SearchEngine + Send + Sync + 'static>
{
    filter: Box<dyn Fn(&FHIRRequest) -> bool + Send + Sync>,
    middleware: Middleware<
        Arc<ClientState<Repo, Search>>,
        ServerCTX,
        FHIRRequest,
        FHIRResponse,
        OperationOutcomeError,
    >,
}

pub struct FHIRServerClient<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
> {
    state: Arc<ClientState<Repo, Search>>,
    middleware: Middleware<
        Arc<ClientState<Repo, Search>>,
        ServerCTX,
        FHIRRequest,
        FHIRResponse,
        OperationOutcomeError,
    >,
}

fn router_middleware_chain<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
>(
    routes: Arc<Vec<Route<Repo, Search>>>,
) -> MiddlewareChain<
    Arc<ClientState<Repo, Search>>,
    ServerCTX,
    FHIRRequest,
    FHIRResponse,
    OperationOutcomeError,
> {
    Box::new(
        move |state: ServerMiddlewareState<Repo, Search>,
              context: ServerMiddlewareContext,
              next: Option<Arc<ServerMiddlewareNext<Repo, Search>>>|
              -> ServerMiddlewareOutput {
            let routes = routes.clone();
            Box::pin(async move {
                let route = Arc::new(routes.iter().find(|r| (r.filter)(&context.request)).clone());
                match route.as_ref() {
                    Some(route) => {
                        let context = route
                            .middleware
                            .call(state.clone(), context.ctx, context.request)
                            .await?;
                        if let Some(next) = next {
                            next(state, context).await
                        } else {
                            Ok(context)
                        }
                    }
                    None => {
                        if let Some(next) = next {
                            next(state, context).await
                        } else {
                            Ok(context)
                        }
                    }
                }
            })
        },
    )
}

impl<Repo: Repository + Send + Sync + 'static, Search: SearchEngine + Send + Sync + 'static>
    FHIRServerClient<Repo, Search>
{
    pub fn new(repo: Repo, search: Search) -> Self {
        let storage_route = Route {
            filter: Box::new(|req: &FHIRRequest| match req {
                FHIRRequest::Read(_)
                | FHIRRequest::SearchSystem(_)
                | FHIRRequest::SearchType(_)
                | FHIRRequest::Create(_)
                // Add other request types as needed
                => true,
                _ => false,
            }),
            middleware: Middleware::new(vec![Box::new(middleware::storage)]),
        };

        let route_middleware = router_middleware_chain(Arc::new(vec![storage_route]));

        FHIRServerClient {
            state: Arc::new(ClientState {
                repo,
                search: Arc::new(search),
            }),
            middleware: Middleware::new(vec![
                Box::new(middleware::capabilities),
                Box::new(route_middleware),
            ]),
        }
    }
}

impl<Repo: Repository + Send + Sync + 'static, Search: SearchEngine + Send + Sync + 'static>
    FHIRClient<ServerCTX, OperationOutcomeError> for FHIRServerClient<Repo, Search>
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
        ctx: ServerCTX,
        resource_type: oxidized_fhir_model::r4::types::ResourceType,
        parameters: Vec<ParsedParameter>,
    ) -> Result<Vec<oxidized_fhir_model::r4::types::Resource>, OperationOutcomeError> {
        let res = self
            .middleware
            .call(
                self.state.clone(),
                ctx,
                FHIRRequest::SearchType(FHIRSearchTypeRequest {
                    resource_type,
                    parameters,
                }),
            )
            .await?;

        match res.response {
            Some(FHIRResponse::SearchType(search_response)) => Ok(search_response.resources),
            _ => panic!("Unexpected response type"),
        }
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
