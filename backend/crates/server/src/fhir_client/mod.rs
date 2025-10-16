use crate::fhir_client::{
    middleware::{
        ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput,
        ServerMiddlewareState,
    },
    utilities::request_to_resource_type,
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
    types::{Author, AuthorId, AuthorKind, ProjectId, SupportedFHIRVersions, TenantId},
};
use std::sync::{Arc, LazyLock};

mod middleware;
mod utilities;

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

impl ServerCTX {
    pub fn new(
        tenant: TenantId,
        project: ProjectId,
        fhir_version: SupportedFHIRVersions,
        author: Author,
    ) -> Self {
        ServerCTX {
            tenant,
            project,
            fhir_version,
            author,
        }
    }

    pub fn root(tenant: TenantId, project: ProjectId) -> Self {
        ServerCTX {
            tenant,
            project,
            fhir_version: SupportedFHIRVersions::R4,
            author: Author {
                id: AuthorId::System,
                kind: AuthorKind::System,
            },
        }
    }
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

pub struct Route<
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

pub struct RouterMiddleware<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
> {
    routes: Arc<Vec<Route<Repo, Search, Terminology>>>,
}

impl<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
> RouterMiddleware<Repo, Search, Terminology>
{
    pub fn new(routes: Arc<Vec<Route<Repo, Search, Terminology>>>) -> Self {
        RouterMiddleware { routes }
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
    > for RouterMiddleware<Repo, Search, Terminology>
{
    fn call(
        &self,
        state: ServerMiddlewareState<Repo, Search, Terminology>,
        context: ServerMiddlewareContext,
        next: Option<Arc<ServerMiddlewareNext<Repo, Search, Terminology>>>,
    ) -> ServerMiddlewareOutput {
        let routes = self.routes.clone();
        Box::pin(async move {
            let route = routes.iter().find(|r| (r.filter)(&context.request));
            match route {
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
    }
}

static ARTIFACT_TYPES: &[ResourceType] = &[
    ResourceType::ValueSet,
    ResourceType::CodeSystem,
    ResourceType::StructureDefinition,
    ResourceType::SearchParameter,
];

static TENANT_AUTH_TYPES: &[ResourceType] = &[ResourceType::User, ResourceType::Project];
static PROJECT_AUTH_TYPES: &[ResourceType] = &[ResourceType::Membership];

static SPECIAL_TYPES: LazyLock<Vec<ResourceType>> = LazyLock::new(|| {
    [
        &TENANT_AUTH_TYPES[..],
        &PROJECT_AUTH_TYPES[..],
        &ARTIFACT_TYPES[..],
    ]
    .concat()
});

pub struct ServerClientConfig<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
> {
    pub repo: Arc<Repo>,
    pub search: Arc<Search>,
    pub terminology: Arc<Terminology>,
    pub mutate_artifacts: bool,
}

impl<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
> ServerClientConfig<Repo, Search, Terminology>
{
    pub fn new(repo: Arc<Repo>, search: Arc<Search>, terminology: Arc<Terminology>) -> Self {
        ServerClientConfig {
            repo,
            search,
            terminology,
            mutate_artifacts: false,
        }
    }

    pub fn allow_mutate_artifacts(
        repo: Arc<Repo>,
        search: Arc<Search>,
        terminology: Arc<Terminology>,
    ) -> Self {
        Self {
            repo,
            search,
            terminology,
            mutate_artifacts: true,
        }
    }
}

impl<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
> FHIRServerClient<Repo, Search, Terminology>
{
    pub fn new(config: ServerClientConfig<Repo, Search, Terminology>) -> Self {
        let clinical_resources_route = Route {
            filter: Box::new(|req: &FHIRRequest| match req {
                FHIRRequest::InvokeInstance(_)
                | FHIRRequest::InvokeType(_)
                | FHIRRequest::InvokeSystem(_)
                | FHIRRequest::Capabilities => false,
                _ => {
                    if let Some(resource_type) = request_to_resource_type(req) {
                        !SPECIAL_TYPES.contains(&resource_type)
                    } else {
                        true
                    }
                }
            }),
            middleware: Middleware::new(vec![Box::new(middleware::storage::Middleware::new())]),
        };

        let operation_invocation_routes = Route {
            filter: Box::new(|req: &FHIRRequest| match req {
                FHIRRequest::InvokeInstance(_)
                | FHIRRequest::InvokeType(_)
                | FHIRRequest::InvokeSystem(_) => true,
                _ => false,
            }),
            middleware: Middleware::new(vec![Box::new(middleware::operations::Middleware::new())]),
        };

        let artifact_routes = Route {
            filter: if config.mutate_artifacts {
                Box::new(|req: &FHIRRequest| match req {
                    FHIRRequest::UpdateInstance(_)
                    | FHIRRequest::ConditionalUpdate(_)
                    | FHIRRequest::Read(_)
                    | FHIRRequest::SearchType(_) => {
                        if let Some(resource_type) = request_to_resource_type(req) {
                            ARTIFACT_TYPES.contains(&resource_type)
                        } else {
                            false
                        }
                    }
                    _ => false,
                })
            } else {
                Box::new(|req: &FHIRRequest| match req {
                    FHIRRequest::Read(_) | FHIRRequest::SearchType(_) => {
                        if let Some(resource_type) = request_to_resource_type(req) {
                            ARTIFACT_TYPES.contains(&resource_type)
                        } else {
                            false
                        }
                    }
                    _ => false,
                })
            },
            middleware: Middleware::new(vec![
                Box::new(middleware::set_artifact_tenant::Middleware::new()),
                Box::new(middleware::storage::Middleware::new()),
            ]),
        };

        let tenant_auth_routes = Route {
            filter: Box::new(|req: &FHIRRequest| match req {
                FHIRRequest::InvokeInstance(_)
                | FHIRRequest::InvokeType(_)
                | FHIRRequest::InvokeSystem(_) => false,
                _ => {
                    request_to_resource_type(req).map_or(false, |rt| TENANT_AUTH_TYPES.contains(rt))
                }
            }),
            middleware: Middleware::new(vec![
                Box::new(middleware::check_project::Middleware::new(
                    ProjectId::System,
                )),
                Box::new(middleware::transaction::Middleware::new()),
                Box::new(middleware::custom_models::project::Middleware::new()),
                Box::new(middleware::custom_models::user::Middleware::new()),
                Box::new(middleware::storage::Middleware::new()),
            ]),
        };

        let route_middleware = RouterMiddleware::new(Arc::new(vec![
            clinical_resources_route,
            artifact_routes,
            operation_invocation_routes,
            // Special Authentication routes.
            tenant_auth_routes,
        ]));

        FHIRServerClient {
            state: Arc::new(ClientState {
                repo: config.repo,
                search: config.search,
                terminology: config.terminology,
            }),
            middleware: Middleware::new(vec![
                Box::new(middleware::capabilities::Middleware::new()),
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
            _ => panic!("Unexpected response type {:?}", res.response),
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
