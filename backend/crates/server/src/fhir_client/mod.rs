use crate::{
    ServerEnvironmentVariables,
    fhir_client::{
        middleware::{
            ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput,
            ServerMiddlewareState,
        },
        utilities::request_to_resource_type,
    },
};
use haste_config::Config;
use haste_fhir_client::{
    FHIRClient,
    middleware::{Middleware, MiddlewareChain},
    request::{
        FHIRBatchRequest, FHIRConditionalUpdateRequest, FHIRCreateRequest, FHIRReadRequest,
        FHIRRequest, FHIRResponse, FHIRSearchTypeRequest, FHIRTransactionRequest,
        FHIRUpdateInstanceRequest, SearchRequest, SearchResponse, UpdateRequest,
    },
    url::ParsedParameters,
};
use haste_fhir_model::r4::generated::resources::{
    Bundle, CapabilityStatement, Parameters, Resource, ResourceType,
};
use haste_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{
    AuthorId, AuthorKind, ProjectId, TenantId, UserRole,
    claims::SubscriptionTier,
    scopes::{
        SMARTResourceScope, Scope, Scopes, SmartResourceScopeLevel, SmartResourceScopePermission,
        SmartResourceScopePermissions, SmartResourceScopeUser, SmartScope,
    },
};
use haste_repository::{Repository, types::SupportedFHIRVersions};
use std::sync::{Arc, LazyLock};

mod batch_transaction_processing;
mod compartment;
mod middleware;
mod resolver;
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

pub struct ServerCTX<Client: FHIRClient<Arc<Self>, OperationOutcomeError>> {
    pub tenant: TenantId,
    pub project: ProjectId,
    pub fhir_version: SupportedFHIRVersions,
    pub user: Arc<haste_jwt::claims::UserTokenClaims>,
    pub client: Arc<Client>,
    pub rate_limit: Arc<dyn haste_rate_limit::RateLimit>,
}

impl<Client: FHIRClient<Arc<Self>, OperationOutcomeError>> ServerCTX<Client> {
    pub fn swap_client<NewClient: FHIRClient<Arc<ServerCTX<NewClient>>, OperationOutcomeError>>(
        &self,
        new_client: Arc<NewClient>,
    ) -> ServerCTX<NewClient> {
        ServerCTX {
            tenant: self.tenant.clone(),
            project: self.project.clone(),
            fhir_version: self.fhir_version.clone(),
            user: self.user.clone(),
            client: new_client,
            rate_limit: self.rate_limit.clone(),
        }
    }

    pub fn new(
        tenant: TenantId,
        project: ProjectId,
        fhir_version: SupportedFHIRVersions,
        user: Arc<haste_jwt::claims::UserTokenClaims>,
        client: Arc<Client>,
        rate_limit: Arc<dyn haste_rate_limit::RateLimit>,
    ) -> Self {
        ServerCTX {
            tenant,
            project,
            fhir_version,
            user,
            client,
            rate_limit,
        }
    }

    pub fn system(
        tenant: TenantId,
        project: ProjectId,
        client: Arc<Client>,
        rate_limit: Arc<dyn haste_rate_limit::RateLimit>,
    ) -> Self {
        ServerCTX {
            tenant: tenant.clone(),
            project: project.clone(),
            fhir_version: SupportedFHIRVersions::R4,
            user: Arc::new(haste_jwt::claims::UserTokenClaims {
                sub: AuthorId::System,
                exp: 0,
                aud: AuthorKind::System.to_string(),
                user_role: UserRole::Owner,
                project: Some(project),
                tenant,
                subscription_tier: SubscriptionTier::Unlimited,
                scope: Scopes(vec![Scope::SMART(SmartScope::Resource(
                    SMARTResourceScope {
                        user: SmartResourceScopeUser::System,
                        level: SmartResourceScopeLevel::AllResources,
                        permissions: SmartResourceScopePermissions::new(vec![
                            SmartResourceScopePermission::Create,
                            SmartResourceScopePermission::Read,
                            SmartResourceScopePermission::Update,
                            SmartResourceScopePermission::Delete,
                            SmartResourceScopePermission::Search,
                        ]),
                    },
                ))]),
                user_id: AuthorId::System,
                resource_type: AuthorKind::System,
                access_policy_version_ids: vec![],
                membership: None,
            }),
            client,
            rate_limit,
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
    config: Arc<dyn Config<ServerEnvironmentVariables>>,
}

pub struct Route<State, Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError>> {
    filter: Box<dyn Fn(&FHIRRequest) -> bool + Send + Sync>,
    middleware: Middleware<
        Arc<State>,
        Arc<ServerCTX<Client>>,
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
        Arc<ServerCTX<Self>>,
        FHIRRequest,
        FHIRResponse,
        OperationOutcomeError,
    >,
}

pub struct RouterMiddleware<
    State,
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError>,
> {
    routes: Arc<Vec<Route<State, Client>>>,
}

impl<State, Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError>>
    RouterMiddleware<State, Client>
{
    pub fn new(routes: Arc<Vec<Route<State, Client>>>) -> Self {
        RouterMiddleware { routes }
    }
}

impl<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>
    MiddlewareChain<
        ServerMiddlewareState<Repo, Search, Terminology>,
        Arc<ServerCTX<Client>>,
        FHIRRequest,
        FHIRResponse,
        OperationOutcomeError,
    > for RouterMiddleware<ClientState<Repo, Search, Terminology>, Client>
{
    fn call(
        &self,
        state: ServerMiddlewareState<Repo, Search, Terminology>,
        context: ServerMiddlewareContext<Client>,
        next: Option<
            Arc<ServerMiddlewareNext<Client, ServerMiddlewareState<Repo, Search, Terminology>>>,
        >,
    ) -> ServerMiddlewareOutput<Client> {
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

static TENANT_AUTH_TYPES: &[ResourceType] = &[
    ResourceType::User,
    ResourceType::Project,
    ResourceType::IdentityProvider,
];
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
    pub config: Arc<dyn Config<ServerEnvironmentVariables>>,
}

impl<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
> ServerClientConfig<Repo, Search, Terminology>
{
    pub fn new(
        repo: Arc<Repo>,
        search: Arc<Search>,
        terminology: Arc<Terminology>,
        config: Arc<dyn Config<ServerEnvironmentVariables>>,
    ) -> Self {
        ServerClientConfig {
            repo,
            search,
            terminology,
            mutate_artifacts: false,
            config,
        }
    }

    pub fn allow_mutate_artifacts(
        repo: Arc<Repo>,
        search: Arc<Search>,
        terminology: Arc<Terminology>,
        config: Arc<dyn Config<ServerEnvironmentVariables>>,
    ) -> Self {
        Self {
            repo,
            search,
            terminology,
            config,
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
        let route_middleware = RouterMiddleware::new(Arc::new(vec![
            // Clinical resources.
            Route {
                filter: Box::new(|req: &FHIRRequest| match req {
                    FHIRRequest::Invocation(_) | FHIRRequest::Capabilities => false,
                    _ => {
                        if let Some(resource_type) = request_to_resource_type(req) {
                            !SPECIAL_TYPES.contains(&resource_type)
                        } else {
                            true
                        }
                    }
                }),
                middleware: Middleware::new(vec![Box::new(middleware::storage::Middleware::new())]),
            },
            // Artifact routes.
            Route {
                filter: Box::new(|req: &FHIRRequest| match req {
                    FHIRRequest::Create(_)
                    | FHIRRequest::Update(_)
                    | FHIRRequest::Delete(_)
                    | FHIRRequest::Read(_)
                    | FHIRRequest::Search(SearchRequest::Type(_)) => {
                        if let Some(resource_type) = request_to_resource_type(req) {
                            ARTIFACT_TYPES.contains(&resource_type)
                        } else {
                            false
                        }
                    }
                    _ => false,
                }),

                middleware: Middleware::new(vec![
                    Box::new(middleware::set_artifact_tenant::Middleware::new()),
                    Box::new(middleware::storage::Middleware::new()),
                ]),
            },
            // Operation routes
            Route {
                filter: Box::new(|req: &FHIRRequest| match req {
                    FHIRRequest::Invocation(_) => true,
                    _ => false,
                }),
                middleware: Middleware::new(vec![Box::new(
                    middleware::operations::Middleware::new(),
                )]),
            },
            // Authentication routes.
            Route {
                filter: Box::new(|req: &FHIRRequest| match req {
                    FHIRRequest::Invocation(_) => false,
                    _ => request_to_resource_type(req)
                        .map_or(false, |rt| PROJECT_AUTH_TYPES.contains(rt)),
                }),
                middleware: Middleware::new(vec![
                    Box::new(middleware::transaction::Middleware::new()),
                    Box::new(middleware::custom_models::membership::Middleware::new()),
                    Box::new(middleware::storage::Middleware::new()),
                ]),
            },
            // Tenant auth routes.
            Route {
                filter: Box::new(|req: &FHIRRequest| match req {
                    FHIRRequest::Invocation(_) => false,
                    _ => request_to_resource_type(req)
                        .map_or(false, |rt| TENANT_AUTH_TYPES.contains(rt)),
                }),
                middleware: Middleware::new(vec![
                    Box::new(
                        middleware::check_project::SetProjectReadOnlyMiddleware::new(
                            ProjectId::System,
                        ),
                    ),
                    // Confirm in system project as above will only set to system if readonly.
                    Box::new(middleware::check_project::Middleware::new(
                        ProjectId::System,
                    )),
                    Box::new(middleware::transaction::Middleware::new()),
                    Box::new(middleware::custom_models::project::Middleware::new()),
                    Box::new(middleware::custom_models::user::Middleware::new()),
                    Box::new(middleware::storage::Middleware::new()),
                ]),
            },
        ]));

        FHIRServerClient {
            state: Arc::new(ClientState {
                repo: config.repo,
                search: config.search,
                terminology: config.terminology,
                config: config.config,
            }),
            middleware: Middleware::new(vec![
                Box::new(middleware::rate_limit::Middleware::new()),
                Box::new(middleware::auth_z::scope_check::SMARTScopeAccessMiddleware::new()),
                Box::new(middleware::auth_z::access_control::AccessControlMiddleware::new()),
                Box::new(middleware::validation::Middleware::new()),
                Box::new(route_middleware),
                Box::new(middleware::capabilities::Middleware::new()),
            ]),
        }
    }
}

impl<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
> FHIRClient<Arc<ServerCTX<Self>>, OperationOutcomeError>
    for FHIRServerClient<Repo, Search, Terminology>
{
    async fn request(
        &self,
        _ctx: Arc<ServerCTX<Self>>,
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
        _ctx: Arc<ServerCTX<Self>>,
    ) -> Result<CapabilityStatement, OperationOutcomeError> {
        let res = self
            .middleware
            .call(self.state.clone(), _ctx, FHIRRequest::Capabilities)
            .await?;

        match res.response {
            Some(FHIRResponse::Capabilities(capabilities_response)) => {
                Ok(capabilities_response.capabilities)
            }
            _ => panic!("Unexpected response type"),
        }
    }

    async fn search_system(
        &self,
        _ctx: Arc<ServerCTX<Self>>,
        _parameters: ParsedParameters,
    ) -> Result<Bundle, OperationOutcomeError> {
        todo!()
    }

    async fn search_type(
        &self,
        ctx: Arc<ServerCTX<Self>>,
        resource_type: ResourceType,
        parameters: ParsedParameters,
    ) -> Result<Bundle, OperationOutcomeError> {
        let res = self
            .middleware
            .call(
                self.state.clone(),
                ctx,
                FHIRRequest::Search(SearchRequest::Type(FHIRSearchTypeRequest {
                    resource_type,
                    parameters,
                })),
            )
            .await?;

        match res.response {
            Some(FHIRResponse::Search(SearchResponse::Type(search_response))) => {
                Ok(search_response.bundle)
            }
            _ => panic!("Unexpected response type"),
        }
    }

    async fn create(
        &self,
        ctx: Arc<ServerCTX<Self>>,
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
        ctx: Arc<ServerCTX<Self>>,
        resource_type: ResourceType,
        id: String,
        resource: Resource,
    ) -> Result<Resource, OperationOutcomeError> {
        let res = self
            .middleware
            .call(
                self.state.clone(),
                ctx,
                FHIRRequest::Update(UpdateRequest::Instance(FHIRUpdateInstanceRequest {
                    resource_type,
                    id,
                    resource,
                })),
            )
            .await?;

        match res.response {
            Some(FHIRResponse::Create(create_response)) => Ok(create_response.resource),
            Some(FHIRResponse::Update(update_response)) => Ok(update_response.resource),
            _ => panic!("Unexpected response type {:?}", res.response),
        }
    }

    async fn conditional_update(
        &self,
        ctx: Arc<ServerCTX<Self>>,
        resource_type: ResourceType,
        parameters: ParsedParameters,
        resource: Resource,
    ) -> Result<Resource, OperationOutcomeError> {
        let res = self
            .middleware
            .call(
                self.state.clone(),
                ctx,
                FHIRRequest::Update(UpdateRequest::Conditional(FHIRConditionalUpdateRequest {
                    resource_type,
                    parameters,
                    resource,
                })),
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
        _ctx: Arc<ServerCTX<Self>>,
        _resource_type: ResourceType,
        _id: String,
        _patches: json_patch::Patch,
    ) -> Result<Resource, OperationOutcomeError> {
        todo!()
    }

    async fn read(
        &self,
        ctx: Arc<ServerCTX<Self>>,
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
            Some(FHIRResponse::Read(read_response)) => Ok(read_response.resource),
            _ => panic!("Unexpected response type"),
        }
    }

    async fn vread(
        &self,
        _ctx: Arc<ServerCTX<Self>>,
        _resource_type: ResourceType,
        _id: String,
        _version_id: String,
    ) -> Result<Option<Resource>, OperationOutcomeError> {
        todo!()
    }

    async fn delete_instance(
        &self,
        _ctx: Arc<ServerCTX<Self>>,
        _resource_type: ResourceType,
        _id: String,
    ) -> Result<(), OperationOutcomeError> {
        todo!()
    }

    async fn delete_type(
        &self,
        _ctx: Arc<ServerCTX<Self>>,
        _resource_type: ResourceType,
        _parameters: ParsedParameters,
    ) -> Result<(), OperationOutcomeError> {
        todo!()
    }

    async fn delete_system(
        &self,
        _ctx: Arc<ServerCTX<Self>>,
        _parameters: ParsedParameters,
    ) -> Result<(), OperationOutcomeError> {
        todo!()
    }

    async fn history_system(
        &self,
        _ctx: Arc<ServerCTX<Self>>,
        _parameters: ParsedParameters,
    ) -> Result<Bundle, OperationOutcomeError> {
        todo!()
    }

    async fn history_type(
        &self,
        _ctx: Arc<ServerCTX<Self>>,
        _resource_type: ResourceType,
        _parameters: ParsedParameters,
    ) -> Result<Bundle, OperationOutcomeError> {
        todo!()
    }

    async fn history_instance(
        &self,
        _ctx: Arc<ServerCTX<Self>>,
        _resource_type: ResourceType,
        _id: String,
        _parameters: ParsedParameters,
    ) -> Result<Bundle, OperationOutcomeError> {
        todo!()
    }

    async fn invoke_instance(
        &self,
        _ctx: Arc<ServerCTX<Self>>,
        _resource_type: ResourceType,
        _id: String,
        _operation: String,
        _parameters: Parameters,
    ) -> Result<Resource, OperationOutcomeError> {
        todo!()
    }

    async fn invoke_type(
        &self,
        _ctx: Arc<ServerCTX<Self>>,
        _resource_type: ResourceType,
        _operation: String,
        _parameters: Parameters,
    ) -> Result<Resource, OperationOutcomeError> {
        todo!()
    }

    async fn invoke_system(
        &self,
        _ctx: Arc<ServerCTX<Self>>,
        _operation: String,
        _parameters: Parameters,
    ) -> Result<Resource, OperationOutcomeError> {
        todo!()
    }

    async fn transaction(
        &self,
        ctx: Arc<ServerCTX<Self>>,
        bundle: Bundle,
    ) -> Result<Bundle, OperationOutcomeError> {
        let res = self
            .middleware
            .call(
                self.state.clone(),
                ctx,
                FHIRRequest::Transaction(FHIRTransactionRequest { resource: bundle }),
            )
            .await?;

        match res.response {
            Some(FHIRResponse::Transaction(transaction_response)) => {
                Ok(transaction_response.resource)
            }
            _ => panic!("Unexpected response type"),
        }
    }

    async fn batch(
        &self,
        ctx: Arc<ServerCTX<Self>>,
        bundle: Bundle,
    ) -> Result<Bundle, OperationOutcomeError> {
        let res = self
            .middleware
            .call(
                self.state.clone(),
                ctx,
                FHIRRequest::Batch(FHIRBatchRequest { resource: bundle }),
            )
            .await?;

        match res.response {
            Some(FHIRResponse::Batch(batch_response)) => Ok(batch_response.resource),
            _ => panic!("Unexpected response type"),
        }
    }
}
