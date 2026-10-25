use crate::{
    auth_n::middleware::jwt::User,
    config::ServerConfig,
    fhir_client::{
        middleware::{Backend, Handler, Output, Pipeline},
        utilities::request_to_resource_type,
    },
};
use derivative::Derivative;
use haste_fhir_client::{
    FHIRClient,
    request::{
        DeleteRequest, DeleteResponse, FHIRBatchRequest, FHIRConditionalUpdateRequest,
        FHIRCreateRequest, FHIRDeleteInstanceRequest, FHIRDeleteSystemRequest,
        FHIRDeleteTypeRequest, FHIRHistoryInstanceRequest, FHIRHistorySystemRequest,
        FHIRHistoryTypeRequest, FHIRInvokeInstanceRequest, FHIRInvokeSystemRequest,
        FHIRInvokeTypeRequest, FHIRPatchRequest, FHIRReadRequest, FHIRRequest, FHIRResponse,
        FHIRSearchSystemRequest, FHIRSearchTypeRequest, FHIRTransactionRequest,
        FHIRUpdateInstanceRequest, FHIRVersionReadRequest, HistoryRequest, HistoryResponse,
        InvocationRequest, InvokeResponse, Operation, SearchRequest, SearchResponse, UpdateRequest,
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
use middleware::{
    auditing::Auditing,
    auth_z::{access_control::AccessControl, scope_check::SmartScopeAccess},
    capabilities::Capabilities,
    check_project::{CheckProject, SetProjectReadOnly},
    custom_models::{
        access_policy_v3::AccessPolicyV3Validation,
        deidentification_policy::DeidentificationPolicyValidation, membership::MembershipSync,
        project::ProjectSync, user::UserSync,
    },
    element_filtering::ElementFiltering,
    operation_definitions::{Builtins, OperationDefinitions},
    operations::Operations,
    patch_as_update::PatchAsUpdate,
    rate_limit::RateLimit,
    set_artifact_tenant::SetArtifactTenant,
    storage::Storage,
    tenant_tier_limits::TenantTierLimits,
    transaction::Transaction,
    validation::Validation,
};
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
    #[error(
        code = "invalid",
        diagnostic = "The resource's type '{arg0}' does not match the requested type '{arg1}'."
    )]
    TypeMismatch(String, String),
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
    /// The request asked for `Prefer: respond-async`: an operation it invokes
    /// is queued as a job and answered with `202 Accepted` instead of run in
    /// the request. Other interactions ignore it, as the pattern allows.
    pub prefer_async: bool,
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
            prefer_async: self.prefer_async,
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
            prefer_async: self.prefer_async,
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
            prefer_async: false,
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
            prefer_async: false,
        }
    }

    pub fn with_tracing_id(mut self, tracing_id: Option<String>) -> Self {
        self.tracing_id = tracing_id;
        self
    }

    #[must_use]
    pub fn with_prefer_async(mut self, prefer_async: bool) -> Self {
        self.prefer_async = prefer_async;
        self
    }
}

/// What every layer runs on. The transaction middleware swaps the repository
/// for a transaction's; the rest is fixed for the client's lifetime.
pub struct ClientState<B: Backend> {
    repo: Arc<B::Repo>,
    search: Arc<B::Search>,
    terminology: Arc<B::Terminology>,
    audit_repo: Option<Arc<B::Repo>>,
    config: Arc<ServerConfig>,
    deno_pool: Arc<DenoPool>,
}

impl<B: Backend> ClientState<B> {
    /// This state reading and writing through `repo`.
    fn with_repo(&self, repo: Arc<B::Repo>) -> Self {
        ClientState {
            repo,
            search: self.search.clone(),
            terminology: self.terminology.clone(),
            audit_repo: self.audit_repo.clone(),
            config: self.config.clone(),
            deno_pool: self.deno_pool.clone(),
        }
    }
}

impl<B: Backend> Clone for ClientState<B> {
    fn clone(&self) -> Self {
        self.with_repo(self.repo.clone())
    }
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
    AccessPolicy,
    DeidentificationPolicy,
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
            ResourceType::AccessPolicyV3 => Lane::AccessPolicy,
            ResourceType::DeidentificationPolicy => Lane::DeidentificationPolicy,
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

/// Rejects a create or update whose body is another type than the request
/// names. Access control, the lane and storage all go by the requested type,
/// so a body of another type would be checked as one type and stored as
/// another.
fn check_body_type(request: &FHIRRequest) -> Result<(), StorageError> {
    let (requested, resource) = match request {
        FHIRRequest::Create(create) => (&create.resource_type, &create.resource),
        FHIRRequest::Update(UpdateRequest::Instance(update)) => {
            (&update.resource_type, &update.resource)
        }
        FHIRRequest::Update(UpdateRequest::Conditional(update)) => {
            (&update.resource_type, &update.resource)
        }
        _ => return Ok(()),
    };

    let actual = resource.resource_type();
    if &actual == requested {
        Ok(())
    } else {
        Err(StorageError::TypeMismatch(
            actual.as_ref().to_string(),
            requested.as_ref().to_string(),
        ))
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

/// Ends the cross-cutting pipeline: runs the request's lane.
struct Router<B: Backend> {
    clinical: Pipeline<B>,
    artifact: Pipeline<B>,
    operation_definition: Pipeline<B>,
    project_auth: Pipeline<B>,
    tenant_auth: Pipeline<B>,
    access_policy: Pipeline<B>,
    deidentification_policy: Pipeline<B>,
    operation: Pipeline<B>,
    capabilities: Pipeline<B>,
}

impl<B: Backend> Handler<B> for Router<B> {
    fn handle<'a>(
        &'a self,
        state: &'a ClientState<B>,
        ctx: Arc<ServerCTX<B>>,
        request: FHIRRequest,
    ) -> Output<'a> {
        match route(&request) {
            Ok(Lane::Clinical) => &self.clinical,
            Ok(Lane::Artifact) => &self.artifact,
            Ok(Lane::OperationDefinition) => &self.operation_definition,
            Ok(Lane::ProjectAuth) => &self.project_auth,
            Ok(Lane::TenantAuth) => &self.tenant_auth,
            Ok(Lane::AccessPolicy) => &self.access_policy,
            Ok(Lane::DeidentificationPolicy) => &self.deidentification_policy,
            Ok(Lane::Operation) => &self.operation,
            Ok(Lane::Capabilities) => &self.capabilities,
            Err(error) => return Box::pin(std::future::ready(Err(error.into()))),
        }
        .run(state, ctx, request)
    }
}

pub struct FHIRServerClient<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
> {
    state: ClientState<Self>,
    pipeline: Pipeline<Self>,
    builtin_operations: Builtins,
}

impl<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
> Backend for FHIRServerClient<Repo, Search, Terminology>
{
    type Repo = Repo;
    type Search = Search;
    type Terminology = Terminology;

    fn builtin_operations(&self) -> &Builtins {
        &self.builtin_operations
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
        let operations = Operations::new(config.deno_pool.clone());
        let builtin_operations = operations.operation_definitions();

        let router = Router {
            clinical: Pipeline::new(vec![], Storage),
            artifact: Pipeline::new(vec![Box::new(SetArtifactTenant)], Storage),
            // OperationDefinitions, served beside the built-in operations' own.
            operation_definition: Pipeline::new(
                vec![
                    // A PATCH runs as an update, so the patched resource gets
                    // the built-in collision checks.
                    Box::new(PatchAsUpdate),
                    Box::new(OperationDefinitions::new(builtin_operations.clone())),
                ],
                Storage,
            ),
            project_auth: Pipeline::new(
                vec![
                    Box::new(Transaction),
                    // A PATCH runs as an update, so the membership row stays in sync.
                    Box::new(PatchAsUpdate),
                    Box::new(MembershipSync),
                ],
                Storage,
            ),
            tenant_auth: Pipeline::new(
                vec![
                    Box::new(SetProjectReadOnly::new(ProjectId::System)),
                    // Confirm in system project as above will only set to system if readonly.
                    Box::new(CheckProject::new(ProjectId::System)),
                    Box::new(Transaction),
                    // A PATCH runs as an update, so it gets the Project update
                    // checks and the users table stays in sync.
                    Box::new(PatchAsUpdate),
                    Box::new(ProjectSync),
                    Box::new(UserSync),
                ],
                Storage,
            ),
            access_policy: Pipeline::new(
                vec![
                    // A PATCH runs as an update, so the patched policy is checked too.
                    Box::new(PatchAsUpdate),
                    Box::new(AccessPolicyV3Validation),
                ],
                Storage,
            ),
            deidentification_policy: Pipeline::new(
                vec![
                    Box::new(PatchAsUpdate),
                    Box::new(DeidentificationPolicyValidation),
                ],
                Storage,
            ),
            operation: Pipeline::new(vec![], operations),
            capabilities: Pipeline::new(vec![], Capabilities),
        };

        FHIRServerClient {
            state: ClientState {
                repo: config.repo,
                audit_repo: config.audit_repo,
                search: config.search,
                terminology: config.terminology,
                config: config.config,
                deno_pool: config.deno_pool,
            },
            pipeline: Pipeline::new(
                vec![
                    Box::new(Auditing),
                    Box::new(TenantTierLimits),
                    Box::new(RateLimit),
                    Box::new(ElementFiltering),
                    Box::new(SmartScopeAccess),
                    Box::new(AccessControl),
                    Box::new(Validation),
                ],
                router,
            ),
            builtin_operations,
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

    /// Runs `request` through the pipeline.
    fn run(&self, ctx: Arc<ServerCTX<Self>>, request: FHIRRequest) -> Output<'_> {
        if let Err(error) = check_body_type(&request) {
            return Box::pin(async move { Err(error.into()) });
        }
        self.pipeline.run(&self.state, ctx, request)
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
        self.run(_ctx, request).await
    }

    async fn capabilities(
        &self,
        _ctx: Arc<ServerCTX<Self>>,
    ) -> Result<CapabilityStatement, OperationOutcomeError> {
        match self.run(_ctx, FHIRRequest::Capabilities).await? {
            FHIRResponse::Capabilities(response) => Ok(response.capabilities),
            _ => Err(StorageError::UnexpectedResponse.into()),
        }
    }

    async fn search_system(
        &self,
        ctx: Arc<ServerCTX<Self>>,
        parameters: ParsedParameters,
    ) -> Result<Bundle, OperationOutcomeError> {
        let request = FHIRRequest::Search(SearchRequest::System(FHIRSearchSystemRequest {
            parameters,
        }));
        match self.run(ctx, request).await? {
            FHIRResponse::Search(SearchResponse::System(response)) => Ok(response.bundle),
            _ => Err(StorageError::UnexpectedResponse.into()),
        }
    }

    async fn search_type(
        &self,
        ctx: Arc<ServerCTX<Self>>,
        resource_type: ResourceType,
        parameters: ParsedParameters,
    ) -> Result<Bundle, OperationOutcomeError> {
        let request = FHIRRequest::Search(SearchRequest::Type(FHIRSearchTypeRequest {
            resource_type,
            parameters,
        }));
        match self.run(ctx, request).await? {
            FHIRResponse::Search(SearchResponse::Type(response)) => Ok(response.bundle),
            _ => Err(StorageError::UnexpectedResponse.into()),
        }
    }

    async fn create(
        &self,
        ctx: Arc<ServerCTX<Self>>,
        resource_type: ResourceType,
        resource: Resource,
    ) -> Result<Resource, OperationOutcomeError> {
        let request = FHIRRequest::Create(FHIRCreateRequest {
            resource_type,
            resource,
        });
        match self.run(ctx, request).await? {
            FHIRResponse::Create(response) => Ok(response.resource),
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
        let request = FHIRRequest::Update(UpdateRequest::Instance(FHIRUpdateInstanceRequest {
            resource_type,
            id,
            resource,
        }));
        match self.run(ctx, request).await? {
            FHIRResponse::Create(response) => Ok(response.resource),
            FHIRResponse::Update(response) => Ok(response.resource),
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
        let request =
            FHIRRequest::Update(UpdateRequest::Conditional(FHIRConditionalUpdateRequest {
                resource_type,
                parameters,
                resource,
            }));
        match self.run(ctx, request).await? {
            FHIRResponse::Create(response) => Ok(response.resource),
            FHIRResponse::Update(response) => Ok(response.resource),
            _ => Err(StorageError::UnexpectedResponse.into()),
        }
    }

    async fn patch(
        &self,
        ctx: Arc<ServerCTX<Self>>,
        resource_type: ResourceType,
        id: String,
        patches: json_patch::Patch,
    ) -> Result<Resource, OperationOutcomeError> {
        let request = FHIRRequest::Patch(FHIRPatchRequest {
            resource_type,
            id,
            patch: patches,
        });
        match self.run(ctx, request).await? {
            FHIRResponse::Patch(response) => Ok(response.resource),
            _ => Err(StorageError::UnexpectedResponse.into()),
        }
    }

    async fn read(
        &self,
        ctx: Arc<ServerCTX<Self>>,
        resource_type: ResourceType,
        id: String,
    ) -> Result<Option<Resource>, OperationOutcomeError> {
        let request = FHIRRequest::Read(FHIRReadRequest { resource_type, id });
        match self.run(ctx, request).await? {
            FHIRResponse::Read(response) => Ok(response.resource),
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
        let request = FHIRRequest::VersionRead(FHIRVersionReadRequest {
            resource_type,
            id,
            version_id: VersionId::new(version_id),
        });
        match self.run(ctx, request).await? {
            FHIRResponse::VersionRead(response) => Ok(Some(response.resource)),
            _ => Err(StorageError::UnexpectedResponse.into()),
        }
    }

    async fn delete_instance(
        &self,
        ctx: Arc<ServerCTX<Self>>,
        resource_type: ResourceType,
        id: String,
    ) -> Result<(), OperationOutcomeError> {
        let request = FHIRRequest::Delete(DeleteRequest::Instance(FHIRDeleteInstanceRequest {
            resource_type,
            id,
        }));
        match self.run(ctx, request).await? {
            FHIRResponse::Delete(DeleteResponse::Instance(_)) => Ok(()),
            _ => Err(StorageError::UnexpectedResponse.into()),
        }
    }

    async fn delete_type(
        &self,
        ctx: Arc<ServerCTX<Self>>,
        resource_type: ResourceType,
        parameters: ParsedParameters,
    ) -> Result<(), OperationOutcomeError> {
        let request = FHIRRequest::Delete(DeleteRequest::Type(FHIRDeleteTypeRequest {
            resource_type,
            parameters,
        }));
        match self.run(ctx, request).await? {
            FHIRResponse::Delete(DeleteResponse::Type(_)) => Ok(()),
            _ => Err(StorageError::UnexpectedResponse.into()),
        }
    }

    async fn delete_system(
        &self,
        ctx: Arc<ServerCTX<Self>>,
        parameters: ParsedParameters,
    ) -> Result<(), OperationOutcomeError> {
        let request = FHIRRequest::Delete(DeleteRequest::System(FHIRDeleteSystemRequest {
            parameters,
        }));
        match self.run(ctx, request).await? {
            FHIRResponse::Delete(DeleteResponse::System(_)) => Ok(()),
            _ => Err(StorageError::UnexpectedResponse.into()),
        }
    }

    async fn history_system(
        &self,
        ctx: Arc<ServerCTX<Self>>,
        parameters: ParsedParameters,
    ) -> Result<Bundle, OperationOutcomeError> {
        let request = FHIRRequest::History(HistoryRequest::System(FHIRHistorySystemRequest {
            parameters,
        }));
        match self.run(ctx, request).await? {
            FHIRResponse::History(HistoryResponse::System(response)) => Ok(response.bundle),
            _ => Err(StorageError::UnexpectedResponse.into()),
        }
    }

    async fn history_type(
        &self,
        ctx: Arc<ServerCTX<Self>>,
        resource_type: ResourceType,
        parameters: ParsedParameters,
    ) -> Result<Bundle, OperationOutcomeError> {
        let request = FHIRRequest::History(HistoryRequest::Type(FHIRHistoryTypeRequest {
            resource_type,
            parameters,
        }));
        match self.run(ctx, request).await? {
            FHIRResponse::History(HistoryResponse::Type(response)) => Ok(response.bundle),
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
        let request = FHIRRequest::History(HistoryRequest::Instance(FHIRHistoryInstanceRequest {
            resource_type,
            id,
            parameters,
        }));
        match self.run(ctx, request).await? {
            FHIRResponse::History(HistoryResponse::Instance(response)) => Ok(response.bundle),
            _ => Err(StorageError::UnexpectedResponse.into()),
        }
    }

    async fn invoke_instance(
        &self,
        ctx: Arc<ServerCTX<Self>>,
        resource_type: ResourceType,
        id: String,
        operation: String,
        parameters: Parameters,
    ) -> Result<Resource, OperationOutcomeError> {
        let request =
            FHIRRequest::Invocation(InvocationRequest::Instance(FHIRInvokeInstanceRequest {
                operation: Operation::new(&operation),
                resource_type,
                id,
                parameters,
            }));
        match self.run(ctx, request).await? {
            FHIRResponse::Invoke(InvokeResponse::Instance(response)) => Ok(response.resource),
            _ => Err(StorageError::UnexpectedResponse.into()),
        }
    }

    async fn invoke_type(
        &self,
        ctx: Arc<ServerCTX<Self>>,
        resource_type: ResourceType,
        operation: String,
        parameters: Parameters,
    ) -> Result<Resource, OperationOutcomeError> {
        let request = FHIRRequest::Invocation(InvocationRequest::Type(FHIRInvokeTypeRequest {
            operation: Operation::new(&operation),
            resource_type,
            parameters,
        }));
        match self.run(ctx, request).await? {
            FHIRResponse::Invoke(InvokeResponse::Type(response)) => Ok(response.resource),
            _ => Err(StorageError::UnexpectedResponse.into()),
        }
    }

    async fn invoke_system(
        &self,
        ctx: Arc<ServerCTX<Self>>,
        operation: String,
        parameters: Parameters,
    ) -> Result<Resource, OperationOutcomeError> {
        let request = FHIRRequest::Invocation(InvocationRequest::System(FHIRInvokeSystemRequest {
            operation: Operation::new(&operation),
            parameters,
        }));
        match self.run(ctx, request).await? {
            FHIRResponse::Invoke(InvokeResponse::System(response)) => Ok(response.resource),
            _ => Err(StorageError::UnexpectedResponse.into()),
        }
    }

    async fn transaction(
        &self,
        ctx: Arc<ServerCTX<Self>>,
        bundle: Bundle,
    ) -> Result<Bundle, OperationOutcomeError> {
        let request = FHIRRequest::Transaction(FHIRTransactionRequest { resource: bundle });
        match self.run(ctx, request).await? {
            FHIRResponse::Transaction(response) => Ok(response.resource),
            _ => Err(StorageError::UnexpectedResponse.into()),
        }
    }

    async fn batch(
        &self,
        ctx: Arc<ServerCTX<Self>>,
        bundle: Bundle,
    ) -> Result<Bundle, OperationOutcomeError> {
        let request = FHIRRequest::Batch(FHIRBatchRequest { resource: bundle });
        match self.run(ctx, request).await? {
            FHIRResponse::Batch(response) => Ok(response.resource),
            _ => Err(StorageError::UnexpectedResponse.into()),
        }
    }
}
