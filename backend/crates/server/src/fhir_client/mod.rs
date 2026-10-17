use crate::{
    auth_n::middleware::jwt::User,
    config::ServerConfig,
    fhir_client::{
        middleware::{
            ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput,
            ServerMiddlewareState,
        },
        utilities::request_to_resource_type,
    },
};
use derivative::Derivative;
use haste_fhir_client::{
    FHIRClient,
    middleware::{Middleware, MiddlewareChain},
    request::{
        DeleteRequest, DeleteResponse, FHIRBatchRequest, FHIRConditionalUpdateRequest,
        FHIRCreateRequest, FHIRDeleteInstanceRequest, FHIRDeleteSystemRequest,
        FHIRDeleteTypeRequest, FHIRHistoryInstanceRequest, FHIRHistorySystemRequest,
        FHIRHistoryTypeRequest, FHIRReadRequest, FHIRRequest, FHIRResponse,
        FHIRSearchSystemRequest, FHIRSearchTypeRequest, FHIRTransactionRequest,
        FHIRUpdateInstanceRequest, FHIRVersionReadRequest, HistoryRequest, HistoryResponse,
        SearchRequest, SearchResponse, UpdateRequest,
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
    AuthorId, AuthorKind, ProjectId, TenantId, UserRole, VersionId,
    claims::SubscriptionTier,
    scopes::{
        SMARTResourceScope, Scope, Scopes, SmartResourceScopeLevel, SmartResourceScopePermission,
        SmartResourceScopePermissions, SmartResourceScopeUser, SmartScope,
    },
};
use haste_operation_executor::providers::deno_embedded::pool::DenoPool;
use haste_repository::{Repository, types::SupportedFHIRVersions};
use std::{fmt::Debug, sync::Arc};

mod batch_transaction_processing;
mod compartment;
mod middleware;
mod resolver;
mod subscription_limits;
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
        code = "exception",
        diagnostic = "The request returned a response of another kind."
    )]
    UnexpectedResponse,
    #[error(
        code = "not-found",
        diagnostic = "Resource '{arg0:?}' with id '{arg1}' not found."
    )]
    NotFound(ResourceType, String),
    #[error(code = "invalid", diagnostic = "Invalid resource type.")]
    InvalidType,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct ServerCTX<Client: FHIRClient<Arc<Self>, OperationOutcomeError>> {
    pub tenant: TenantId,
    pub project: ProjectId,
    pub fhir_version: SupportedFHIRVersions,
    pub user: Arc<User>,
    #[derivative(Debug = "ignore")]
    pub client: Arc<Client>,
    #[derivative(Debug = "ignore")]
    pub rate_limit: Arc<dyn haste_rate_limit::RateLimit>,
    pub tracing_id: Option<String>,
    /// The Patient whose compartment the caller is confined to, set by the
    /// scope middleware when a `patient/` scope grants the request. Storage
    /// filters every search by it.
    pub compartment: Option<String>,
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
            tracing_id: self.tracing_id.clone(),
            compartment: self.compartment.clone(),
        }
    }

    /// This context confined to `patient_id`'s compartment.
    #[must_use]
    pub fn with_compartment(&self, patient_id: String) -> Self {
        ServerCTX {
            tenant: self.tenant.clone(),
            project: self.project.clone(),
            fhir_version: self.fhir_version.clone(),
            user: self.user.clone(),
            client: self.client.clone(),
            rate_limit: self.rate_limit.clone(),
            tracing_id: self.tracing_id.clone(),
            compartment: Some(patient_id),
        }
    }

    pub fn new(
        tenant: TenantId,
        project: ProjectId,
        fhir_version: SupportedFHIRVersions,
        user: Arc<User>,
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
            tracing_id: None,
            compartment: None,
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
            user: Arc::new(User {
                token: None,
                claims: haste_jwt::claims::UserTokenClaims {
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
                    fhir_user: None,
                    patient: None,
                    user_id: AuthorId::System,
                    resource_type: AuthorKind::System,
                    access_policy_version_ids: vec![],
                    membership: None,
                    fhir_version: SupportedFHIRVersions::R4,
                },
            }),
            client,
            rate_limit,
            tracing_id: None,
            compartment: None,
        }
    }

    pub fn with_tracing_id(mut self, tracing_id: Option<String>) -> Self {
        self.tracing_id = tracing_id;
        self
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
    audit_repo: Option<Arc<Repo>>,
    config: Arc<ServerConfig>,
    deno_pool: Arc<DenoPool>,
}

/// The pipelines a request is dispatched to after the cross-cutting chain.
/// Each ends in storage, the operation executor or the capability statement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Lane {
    Clinical,
    Artifact,
    OperationDefinition,
    ProjectAuth,
    TenantAuth,
    Operation,
    Capabilities,
}

impl Lane {
    /// The lane that stores `resource_type`.
    fn of(resource_type: &ResourceType) -> Lane {
        match resource_type {
            ResourceType::ValueSet
            | ResourceType::CodeSystem
            | ResourceType::StructureDefinition
            | ResourceType::SearchParameter => Lane::Artifact,
            ResourceType::OperationDefinition => Lane::OperationDefinition,
            ResourceType::Membership => Lane::ProjectAuth,
            ResourceType::User | ResourceType::Project | ResourceType::IdentityProvider => {
                Lane::TenantAuth
            }
            _ => Lane::Clinical,
        }
    }

    /// Whether this lane serves `request`. Artifacts are read from the system
    /// tenant beside the project's, which only these interactions do.
    fn allows(self, request: &FHIRRequest) -> bool {
        match self {
            Lane::Artifact => matches!(
                request,
                FHIRRequest::Create(_)
                    | FHIRRequest::Read(_)
                    | FHIRRequest::Update(_)
                    | FHIRRequest::Patch(_)
                    | FHIRRequest::Delete(_)
                    | FHIRRequest::Search(SearchRequest::Type(_))
            ),
            _ => true,
        }
    }
}

/// The lane for `request`, or `NotSupported` when its lane does not serve
/// the interaction. Every variant is matched: a new one fails to compile here.
fn route(request: &FHIRRequest) -> Result<Lane, StorageError> {
    match request {
        FHIRRequest::Capabilities => Ok(Lane::Capabilities),
        FHIRRequest::Invocation(_) => Ok(Lane::Operation),
        // Bundle entries and compartment searches are re-dispatched through
        // `ctx.client`, so each is routed on its own. System-level requests
        // have no type to route by.
        FHIRRequest::Batch(_)
        | FHIRRequest::Transaction(_)
        | FHIRRequest::Compartment(_)
        | FHIRRequest::Search(SearchRequest::System(_))
        | FHIRRequest::History(HistoryRequest::System(_))
        | FHIRRequest::Delete(DeleteRequest::System(_)) => Ok(Lane::Clinical),
        FHIRRequest::Create(_)
        | FHIRRequest::Read(_)
        | FHIRRequest::VersionRead(_)
        | FHIRRequest::Update(_)
        | FHIRRequest::Patch(_)
        | FHIRRequest::Delete(DeleteRequest::Instance(_) | DeleteRequest::Type(_))
        | FHIRRequest::History(HistoryRequest::Instance(_) | HistoryRequest::Type(_))
        | FHIRRequest::Search(SearchRequest::Type(_)) => {
            let resource_type =
                request_to_resource_type(request).ok_or(StorageError::InvalidType)?;
            let lane = Lane::of(resource_type);
            if lane.allows(request) {
                Ok(lane)
            } else {
                Err(StorageError::NotSupported)
            }
        }
    }
}

type Chain<State, Client> = Middleware<
    Arc<State>,
    Arc<ServerCTX<Client>>,
    FHIRRequest,
    FHIRResponse,
    OperationOutcomeError,
>;

/// Ends the chain: runs the request's lane and returns its response.
struct Router<State, Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError>> {
    clinical: Chain<State, Client>,
    artifact: Chain<State, Client>,
    operation_definition: Chain<State, Client>,
    project_auth: Chain<State, Client>,
    tenant_auth: Chain<State, Client>,
    operation: Chain<State, Client>,
    capabilities: Chain<State, Client>,
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
    > for Router<ClientState<Repo, Search, Terminology>, Client>
{
    fn call(
        &self,
        state: ServerMiddlewareState<Repo, Search, Terminology>,
        context: ServerMiddlewareContext<Client>,
        next: Option<
            Arc<ServerMiddlewareNext<Client, ServerMiddlewareState<Repo, Search, Terminology>>>,
        >,
    ) -> ServerMiddlewareOutput<Client> {
        debug_assert!(next.is_none(), "the router ends the chain");
        match route(&context.request) {
            Ok(Lane::Clinical) => &self.clinical,
            Ok(Lane::Artifact) => &self.artifact,
            Ok(Lane::OperationDefinition) => &self.operation_definition,
            Ok(Lane::ProjectAuth) => &self.project_auth,
            Ok(Lane::TenantAuth) => &self.tenant_auth,
            Ok(Lane::Operation) => &self.operation,
            Ok(Lane::Capabilities) => &self.capabilities,
            Err(error) => return Box::pin(std::future::ready(Err(error.into()))),
        }
        .call(state, context.ctx, context.request)
    }
}

pub struct ServerClientConfig<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
> {
    pub repo: Arc<Repo>,
    pub audit_repo: Option<Arc<Repo>>,
    pub search: Arc<Search>,
    pub terminology: Arc<Terminology>,
    pub mutate_artifacts: bool,
    pub config: Arc<ServerConfig>,
    pub deno_pool: Arc<DenoPool>,
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
        config: Arc<ServerConfig>,
        deno_pool: Arc<DenoPool>,
    ) -> Self {
        ServerClientConfig {
            repo,
            audit_repo: None,
            search,
            terminology,
            mutate_artifacts: false,
            config,
            deno_pool,
        }
    }

    pub fn with_mutate_artifacts(mut self, mutate_artifacts: bool) -> Self {
        self.mutate_artifacts = mutate_artifacts;
        self
    }

    pub fn with_audit_repo(mut self, audit_repo: Option<Arc<Repo>>) -> Self {
        self.audit_repo = audit_repo;
        self
    }
}

impl<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
> FHIRServerClient<Repo, Search, Terminology>
{
    pub fn new(config: ServerClientConfig<Repo, Search, Terminology>) -> Self {
        let operations = middleware::operations::Middleware::new(config.deno_pool.clone());
        let builtin_operations = operations.operation_definitions();

        let router: Router<ClientState<Repo, Search, Terminology>, Self> = Router {
            clinical: Middleware::new(vec![Box::new(middleware::storage::Middleware::new())]),
            artifact: Middleware::new(vec![
                Box::new(middleware::set_artifact_tenant::Middleware::new()),
                Box::new(middleware::storage::Middleware::new()),
            ]),
            // OperationDefinitions, served beside the built-in operations' own.
            operation_definition: Middleware::new(vec![
                // A PATCH runs as an update, so the patched resource gets
                // the built-in collision checks.
                Box::new(middleware::patch_as_update::Middleware::new()),
                Box::new(middleware::operation_definitions::Middleware::new(
                    builtin_operations,
                )),
                Box::new(middleware::storage::Middleware::new()),
            ]),
            project_auth: Middleware::new(vec![
                Box::new(middleware::transaction::Middleware::new()),
                // A PATCH runs as an update, so the membership row stays in sync.
                Box::new(middleware::patch_as_update::Middleware::new()),
                Box::new(middleware::custom_models::membership::Middleware::new()),
                Box::new(middleware::storage::Middleware::new()),
            ]),
            tenant_auth: Middleware::new(vec![
                Box::new(
                    middleware::check_project::SetProjectReadOnlyMiddleware::new(ProjectId::System),
                ),
                // Confirm in system project as above will only set to system if readonly.
                Box::new(middleware::check_project::Middleware::new(
                    ProjectId::System,
                )),
                Box::new(middleware::transaction::Middleware::new()),
                // A PATCH runs as an update, so it gets the Project update
                // checks and the users table stays in sync.
                Box::new(middleware::patch_as_update::Middleware::new()),
                Box::new(middleware::custom_models::project::Middleware::new()),
                Box::new(middleware::custom_models::user::Middleware::new()),
                Box::new(middleware::storage::Middleware::new()),
            ]),
            operation: Middleware::new(vec![Box::new(operations)]),
            capabilities: Middleware::new(vec![Box::new(
                middleware::capabilities::Middleware::new(),
            )]),
        };

        FHIRServerClient {
            state: Arc::new(ClientState {
                repo: config.repo,
                audit_repo: config.audit_repo,
                search: config.search,
                terminology: config.terminology,
                config: config.config,
                deno_pool: config.deno_pool,
            }),
            middleware: Middleware::new(vec![
                Box::new(middleware::auditing::Middleware::new()),
                Box::new(middleware::tenant_tier_limits::Middleware::new()),
                Box::new(middleware::rate_limit::Middleware::new()),
                Box::new(middleware::element_filtering::Middleware::new()),
                Box::new(middleware::auth_z::scope_check::SMARTScopeAccessMiddleware::new()),
                Box::new(middleware::auth_z::access_control::AccessControlMiddleware::new()),
                Box::new(middleware::validation::Middleware::new()),
                Box::new(router),
            ]),
        }
    }

    /// The `DenoPool` backing this client's custom-operations middleware.
    ///
    /// Used to hand the same pool to derived clients (transaction-scoped,
    /// batch-scoped, etc.) instead of each one spinning up its own set of
    /// worker threads.
    pub fn deno_pool(&self) -> Arc<DenoPool> {
        self.state.deno_pool.clone()
    }
}

impl<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
> FHIRClient<Arc<ServerCTX<Self>>, OperationOutcomeError>
    for FHIRServerClient<Repo, Search, Terminology>
{
    #[tracing::instrument(name = "fhir_server_client", skip(self))]
    async fn request(
        &self,
        _ctx: Arc<ServerCTX<Self>>,
        request: FHIRRequest,
    ) -> Result<FHIRResponse, OperationOutcomeError> {
        tracing::info!("Processing request");
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
            _ => Err(StorageError::UnexpectedResponse.into()),
        }
    }

    async fn search_system(
        &self,
        ctx: Arc<ServerCTX<Self>>,
        parameters: ParsedParameters,
    ) -> Result<Bundle, OperationOutcomeError> {
        let res = self
            .middleware
            .call(
                self.state.clone(),
                ctx,
                FHIRRequest::Search(SearchRequest::System(FHIRSearchSystemRequest {
                    parameters,
                })),
            )
            .await?;

        match res.response {
            Some(FHIRResponse::Search(SearchResponse::System(search_response))) => {
                Ok(search_response.bundle)
            }
            _ => Err(StorageError::UnexpectedResponse.into()),
        }
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
            _ => Err(StorageError::UnexpectedResponse.into()),
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
            _ => Err(StorageError::UnexpectedResponse.into()),
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
            _ => Err(StorageError::UnexpectedResponse.into()),
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
            _ => Err(StorageError::UnexpectedResponse.into()),
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
            _ => Err(StorageError::UnexpectedResponse.into()),
        }
    }

    async fn vread(
        &self,
        ctx: Arc<ServerCTX<Self>>,
        resource_type: ResourceType,
        id: String,
        version_id: String,
    ) -> Result<Option<Resource>, OperationOutcomeError> {
        let res = self
            .middleware
            .call(
                self.state.clone(),
                ctx,
                FHIRRequest::VersionRead(FHIRVersionReadRequest {
                    resource_type,
                    id,
                    version_id: VersionId::new(version_id),
                }),
            )
            .await?;

        match res.response {
            Some(FHIRResponse::VersionRead(version_read_response)) => {
                Ok(Some(version_read_response.resource))
            }
            _ => Err(StorageError::UnexpectedResponse.into()),
        }
    }

    async fn delete_instance(
        &self,
        ctx: Arc<ServerCTX<Self>>,
        resource_type: ResourceType,
        id: String,
    ) -> Result<(), OperationOutcomeError> {
        let res = self
            .middleware
            .call(
                self.state.clone(),
                ctx,
                FHIRRequest::Delete(DeleteRequest::Instance(FHIRDeleteInstanceRequest {
                    resource_type,
                    id,
                })),
            )
            .await?;

        match res.response {
            Some(FHIRResponse::Delete(DeleteResponse::Instance(_delete_response))) => Ok(()),
            _ => Err(StorageError::UnexpectedResponse.into()),
        }
    }

    async fn delete_type(
        &self,
        ctx: Arc<ServerCTX<Self>>,
        resource_type: ResourceType,
        parameters: ParsedParameters,
    ) -> Result<(), OperationOutcomeError> {
        let res = self
            .middleware
            .call(
                self.state.clone(),
                ctx,
                FHIRRequest::Delete(DeleteRequest::Type(FHIRDeleteTypeRequest {
                    resource_type,
                    parameters,
                })),
            )
            .await?;

        match res.response {
            Some(FHIRResponse::Delete(DeleteResponse::Type(_delete_response))) => Ok(()),
            _ => Err(StorageError::UnexpectedResponse.into()),
        }
    }

    async fn delete_system(
        &self,
        ctx: Arc<ServerCTX<Self>>,
        parameters: ParsedParameters,
    ) -> Result<(), OperationOutcomeError> {
        let res = self
            .middleware
            .call(
                self.state.clone(),
                ctx,
                FHIRRequest::Delete(DeleteRequest::System(FHIRDeleteSystemRequest {
                    parameters,
                })),
            )
            .await?;

        match res.response {
            Some(FHIRResponse::Delete(DeleteResponse::System(_delete_response))) => Ok(()),
            _ => Err(StorageError::UnexpectedResponse.into()),
        }
    }

    async fn history_system(
        &self,
        ctx: Arc<ServerCTX<Self>>,
        parameters: ParsedParameters,
    ) -> Result<Bundle, OperationOutcomeError> {
        let res = self
            .middleware
            .call(
                self.state.clone(),
                ctx,
                FHIRRequest::History(HistoryRequest::System(FHIRHistorySystemRequest {
                    parameters,
                })),
            )
            .await?;
        match res.response {
            Some(FHIRResponse::History(HistoryResponse::System(history_response))) => {
                Ok(history_response.bundle)
            }
            _ => Err(StorageError::UnexpectedResponse.into()),
        }
    }

    async fn history_type(
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
                FHIRRequest::History(HistoryRequest::Type(FHIRHistoryTypeRequest {
                    resource_type,
                    parameters,
                })),
            )
            .await?;
        match res.response {
            Some(FHIRResponse::History(HistoryResponse::Type(history_response))) => {
                Ok(history_response.bundle)
            }
            _ => Err(StorageError::UnexpectedResponse.into()),
        }
    }

    async fn history_instance(
        &self,
        ctx: Arc<ServerCTX<Self>>,
        resource_type: ResourceType,
        id: String,
        parameters: ParsedParameters,
    ) -> Result<Bundle, OperationOutcomeError> {
        let res = self
            .middleware
            .call(
                self.state.clone(),
                ctx,
                FHIRRequest::History(HistoryRequest::Instance(FHIRHistoryInstanceRequest {
                    resource_type,
                    id,
                    parameters,
                })),
            )
            .await?;
        match res.response {
            Some(FHIRResponse::History(HistoryResponse::Instance(history_response))) => {
                Ok(history_response.bundle)
            }
            _ => Err(StorageError::UnexpectedResponse.into()),
        }
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
            _ => Err(StorageError::UnexpectedResponse.into()),
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
            _ => Err(StorageError::UnexpectedResponse.into()),
        }
    }
}
