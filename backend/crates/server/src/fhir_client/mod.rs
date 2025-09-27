use crate::fhir_client::middleware::{
    ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput, ServerMiddlewareState,
};
use oxidized_fhir_client::{
    FHIRClient,
    middleware::{Middleware, MiddlewareChain},
    request::{
        FHIRConditionalUpdateRequest, FHIRCreateRequest, FHIRReadRequest, FHIRRequest,
        FHIRResponse, FHIRSearchTypeRequest,
    },
    url::ParsedParameter,
};
use oxidized_fhir_model::r4::generated::resources::{
    CapabilityStatement, Parameters, Resource, ResourceType,
};
use oxidized_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
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

struct ClientState<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
> {
    repo: Arc<Repo>,
    search: Arc<Search>,
    terminology: Arc<Terminology>,
}

struct Route<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
> {
    filter: Box<dyn Fn(&FHIRRequest) -> bool + Send + Sync>,
    middleware: Middleware<
        Arc<ClientState<Repo, Search, Terminology>>,
        Arc<ServerCTX>,
        FHIRRequest,
        FHIRResponse,
        OperationOutcomeError,
    >,
}

pub struct FHIRServerClient<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
> {
    state: Arc<ClientState<Repo, Search, Terminology>>,
    middleware: Middleware<
        Arc<ClientState<Repo, Search, Terminology>>,
        Arc<ServerCTX>,
        FHIRRequest,
        FHIRResponse,
        OperationOutcomeError,
    >,
}

fn router_middleware_chain<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    routes: Arc<Vec<Route<Repo, Search, Terminology>>>,
) -> MiddlewareChain<
    Arc<ClientState<Repo, Search, Terminology>>,
    Arc<ServerCTX>,
    FHIRRequest,
    FHIRResponse,
    OperationOutcomeError,
> {
    Box::new(
        move |state: ServerMiddlewareState<Repo, Search, Terminology>,
              context: ServerMiddlewareContext,
              next: Option<Arc<ServerMiddlewareNext<Repo, Search, Terminology>>>|
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

static ARTIFACT_TYPES: &[&str] = &[
    "ValueSet",
    "CodeSystem",
    "StructureDefinition",
    "SearchParameter",
];

fn request_to_resource_type<'a>(request: &'a FHIRRequest) -> Option<&'a ResourceType> {
    match request {
        FHIRRequest::Read(req) => Some(&req.resource_type),
        FHIRRequest::VersionRead(req) => Some(&req.resource_type),
        FHIRRequest::UpdateInstance(req) => Some(&req.resource_type),
        FHIRRequest::DeleteInstance(req) => Some(&req.resource_type),
        FHIRRequest::Patch(req) => Some(&req.resource_type),
        FHIRRequest::HistoryInstance(req) => Some(&req.resource_type),

        // Type operations
        FHIRRequest::Create(req) => Some(&req.resource_type),
        FHIRRequest::HistoryType(req) => Some(&req.resource_type),
        FHIRRequest::SearchType(req) => Some(&req.resource_type),
        FHIRRequest::ConditionalUpdate(req) => Some(&req.resource_type),
        FHIRRequest::DeleteType(req) => Some(&req.resource_type),
        _ => None,
    }
}

impl<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
> FHIRServerClient<Repo, Search, Terminology>
{
    pub fn new(repo: Arc<Repo>, search: Arc<Search>, terminology: Arc<Terminology>) -> Self {
        let storage_route = Route {
            filter: Box::new(|req: &FHIRRequest| match req {
                // Instance Operations
                FHIRRequest::Read(_)
                | FHIRRequest::VersionRead(_)
                | FHIRRequest::UpdateInstance(_)
                | FHIRRequest::DeleteInstance(_)
                | FHIRRequest::Patch(_)
                | FHIRRequest::HistoryInstance(_)

                // Type operations
                | FHIRRequest::Create(_)
                | FHIRRequest::HistoryType(_)
                | FHIRRequest::SearchType(_)
                | FHIRRequest::ConditionalUpdate(_)
                | FHIRRequest::DeleteType(_) => {
                    if let Some(resource_type) = request_to_resource_type(req) {
                       !ARTIFACT_TYPES.contains(&resource_type.as_ref())
                    } else {
                        false
                    }
                }

                // System operations
                | FHIRRequest::HistorySystem(_)
                | FHIRRequest::SearchSystem(_)
                | FHIRRequest::DeleteSystem(_)
                // Bundle operations
                | FHIRRequest::Batch(_)
                | FHIRRequest::Transaction(_)
                => true,

                FHIRRequest::InvokeInstance(_)
                | FHIRRequest::InvokeType(_)
                | FHIRRequest::InvokeSystem(_)
                | FHIRRequest::Capabilities
                 => false,
            }),
            middleware: Middleware::new(vec![Box::new(middleware::storage)]),
        };

        let ops_route = Route {
            filter: Box::new(|req: &FHIRRequest| match req {
                FHIRRequest::InvokeInstance(_)
                | FHIRRequest::InvokeType(_)
                | FHIRRequest::InvokeSystem(_) => true,
                _ => false,
            }),
            middleware: Middleware::new(vec![Box::new(middleware::operations)]),
        };

        let artifact_route = Route {
            filter: Box::new(|req: &FHIRRequest| match req {
                FHIRRequest::Read(_) | FHIRRequest::SearchType(_) => {
                    if let Some(resource_type) = request_to_resource_type(req) {
                        ARTIFACT_TYPES.contains(&resource_type.as_ref())
                    } else {
                        false
                    }
                }
                _ => false,
            }),
            middleware: Middleware::new(vec![
                Box::new(middleware::set_artifact_tenant),
                Box::new(middleware::storage),
            ]),
        };

        let route_middleware =
            router_middleware_chain(Arc::new(vec![storage_route, artifact_route, ops_route]));

        FHIRServerClient {
            state: Arc::new(ClientState {
                repo,
                search,
                terminology,
            }),
            middleware: Middleware::new(vec![
                Box::new(middleware::capabilities),
                Box::new(route_middleware),
            ]),
        }
    }
}

impl<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
> FHIRClient<Arc<ServerCTX>, OperationOutcomeError>
    for FHIRServerClient<Repo, Search, Terminology>
{
    async fn request(
        &self,
        _ctx: Arc<ServerCTX>,
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

    async fn capabilities(&self, _ctx: Arc<ServerCTX>) -> CapabilityStatement {
        todo!()
    }

    async fn search_system(
        &self,
        _ctx: Arc<ServerCTX>,
        _parameters: Vec<ParsedParameter>,
    ) -> Result<Vec<Resource>, OperationOutcomeError> {
        todo!()
    }

    async fn search_type(
        &self,
        ctx: Arc<ServerCTX>,
        resource_type: ResourceType,
        parameters: Vec<ParsedParameter>,
    ) -> Result<Vec<Resource>, OperationOutcomeError> {
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
        ctx: Arc<ServerCTX>,
        resource_type: ResourceType,
        resource: Resource,
    ) -> Result<Resource, OperationOutcomeError> {
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
        _ctx: Arc<ServerCTX>,
        _resource_type: ResourceType,
        _id: String,
        _resource: Resource,
    ) -> Result<Resource, OperationOutcomeError> {
        todo!()
    }

    async fn conditional_update(
        &self,
        ctx: Arc<ServerCTX>,
        resource_type: ResourceType,
        parameters: Vec<ParsedParameter>,
        resource: Resource,
    ) -> Result<Resource, OperationOutcomeError> {
        let res = self
            .middleware
            .call(
                self.state.clone(),
                ctx,
                FHIRRequest::ConditionalUpdate(FHIRConditionalUpdateRequest {
                    resource_type,
                    parameters,
                    resource,
                }),
            )
            .await?;

        match res.response {
            Some(FHIRResponse::Create(create_response)) => Ok(create_response.resource),
            Some(FHIRResponse::Update(update_response)) => Ok(update_response.resource),
            _ => panic!("Unexpected response type"),
        }
    }

    async fn patch(
        &self,
        _ctx: Arc<ServerCTX>,
        _resource_type: ResourceType,
        _id: String,
        _patches: json_patch::Patch,
    ) -> Result<Resource, OperationOutcomeError> {
        todo!()
    }

    async fn read(
        &self,
        ctx: Arc<ServerCTX>,
        resource_type: ResourceType,
        id: String,
    ) -> Result<Option<Resource>, OperationOutcomeError> {
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
        _ctx: Arc<ServerCTX>,
        _resource_type: ResourceType,
        _id: String,
        _version_id: String,
    ) -> Result<Option<Resource>, OperationOutcomeError> {
        todo!()
    }

    async fn delete_instance(
        &self,
        _ctx: Arc<ServerCTX>,
        _resource_type: ResourceType,
        _id: String,
    ) -> Result<(), OperationOutcomeError> {
        todo!()
    }

    async fn delete_type(
        &self,
        _ctx: Arc<ServerCTX>,
        _resource_type: ResourceType,
        _parameters: Vec<ParsedParameter>,
    ) -> Result<(), OperationOutcomeError> {
        todo!()
    }

    async fn delete_system(
        &self,
        _ctx: Arc<ServerCTX>,
        _parameters: Vec<ParsedParameter>,
    ) -> Result<(), OperationOutcomeError> {
        todo!()
    }

    async fn history_system(
        &self,
        _ctx: Arc<ServerCTX>,
        _parameters: Vec<ParsedParameter>,
    ) -> Result<Vec<Resource>, OperationOutcomeError> {
        todo!()
    }

    async fn history_type(
        &self,
        _ctx: Arc<ServerCTX>,
        _resource_type: ResourceType,
        _parameters: Vec<ParsedParameter>,
    ) -> Result<Vec<Resource>, OperationOutcomeError> {
        todo!()
    }

    async fn history_instance(
        &self,
        _ctx: Arc<ServerCTX>,
        _resource_type: ResourceType,
        _id: String,
        _parameters: Vec<ParsedParameter>,
    ) -> Result<Vec<Resource>, OperationOutcomeError> {
        todo!()
    }

    async fn invoke_instance(
        &self,
        _ctx: Arc<ServerCTX>,
        _resource_type: ResourceType,
        _id: String,
        _operation: String,
        _parameters: Parameters,
    ) -> Result<Resource, OperationOutcomeError> {
        todo!()
    }

    async fn invoke_type(
        &self,
        _ctx: Arc<ServerCTX>,
        _resource_type: ResourceType,
        _operation: String,
        _parameters: Parameters,
    ) -> Result<Resource, OperationOutcomeError> {
        todo!()
    }

    async fn invoke_system(
        &self,
        _ctx: Arc<ServerCTX>,
        _operation: String,
        _parameters: Parameters,
    ) -> Result<Resource, OperationOutcomeError> {
        todo!()
    }

    async fn transaction(
        &self,
        _ctx: Arc<ServerCTX>,
        _bundle: Resource,
    ) -> Result<Resource, OperationOutcomeError> {
        todo!()
    }

    async fn batch(
        &self,
        _ctx: Arc<ServerCTX>,
        _bundle: Resource,
    ) -> Result<Resource, OperationOutcomeError> {
        todo!()
    }
}
