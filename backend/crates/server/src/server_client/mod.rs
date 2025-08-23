use oxidized_fhir_client::{
    FHIRClient,
    middleware::{Context, Middleware, MiddlewareOutput, Next},
    request::{
        FHIRConditionalUpdateRequest, FHIRCreateRequest, FHIRCreateResponse,
        FHIRHistoryInstanceResponse, FHIRReadRequest, FHIRReadResponse, FHIRRequest, FHIRResponse,
        FHIRSearchTypeRequest, FHIRSearchTypeResponse, FHIRUpdateResponse, FHIRVersionReadResponse,
    },
    url::ParsedParameter,
};
use oxidized_fhir_model::r4::types::ResourceType;
use oxidized_fhir_operation_error::{
    OperationOutcomeCodes, OperationOutcomeError, derive::OperationOutcomeError,
};
use oxidized_fhir_search::{SearchEngine, SearchRequest};
use oxidized_reflect::MetaValue;
use oxidized_repository::{
    Repository,
    fhir::{FHIRRepository, HistoryRequest},
    types::{Author, ProjectId, ResourceId, SupportedFHIRVersions, TenantId, VersionIdRef},
};
use std::sync::Arc;

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

type ServerMiddlewareState<Repository, Search> = Arc<ClientState<Repository, Search>>;
type ServerMiddlewareContext = Context<ServerCTX, FHIRRequest, FHIRResponse>;
type ServerMiddlewareNext<Repo, Search> =
    Next<Arc<ClientState<Repo, Search>>, ServerMiddlewareContext, OperationOutcomeError>;
type ServerMiddlewareOutput = MiddlewareOutput<ServerMiddlewareContext, OperationOutcomeError>;

fn storage_middleware<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
>(
    state: ServerMiddlewareState<Repo, Search>,
    mut context: ServerMiddlewareContext,
    next: Option<Arc<ServerMiddlewareNext<Repo, Search>>>,
) -> ServerMiddlewareOutput {
    Box::pin(async move {
        let response = match &mut context.request {
            FHIRRequest::Create(create_request) => Some(FHIRResponse::Create(FHIRCreateResponse {
                resource: FHIRRepository::create(
                    &state.repo,
                    &context.ctx.tenant,
                    &context.ctx.project,
                    &context.ctx.author,
                    &context.ctx.fhir_version,
                    &mut create_request.resource,
                )
                .await?,
            })),
            FHIRRequest::Read(read_request) => {
                let resource = state
                    .repo
                    .read_latest(
                        &context.ctx.tenant,
                        &context.ctx.project,
                        &read_request.resource_type,
                        &ResourceId::new(read_request.id.to_string()),
                    )
                    .await?
                    .ok_or_else(|| {
                        StorageError::NotFound(
                            read_request.resource_type.clone(),
                            read_request.id.clone(),
                        )
                    })?;

                Some(FHIRResponse::Read(FHIRReadResponse { resource: resource }))
            }
            FHIRRequest::VersionRead(vread_request) => {
                let mut vread_resources = state
                    .repo
                    .read_by_version_ids(
                        &context.ctx.tenant,
                        &context.ctx.project,
                        vec![VersionIdRef::new(&vread_request.version_id)],
                    )
                    .await?;

                if vread_resources.get(0).is_some() {
                    Some(FHIRResponse::VersionRead(FHIRVersionReadResponse {
                        resource: vread_resources.swap_remove(0),
                    }))
                } else {
                    None
                }
            }
            FHIRRequest::HistoryInstance(history_instance_request) => {
                let history_resources = state
                    .repo
                    .history(
                        &context.ctx.tenant,
                        &context.ctx.project,
                        HistoryRequest::Instance(&history_instance_request),
                    )
                    .await?;

                Some(FHIRResponse::HistoryInstance(FHIRHistoryInstanceResponse {
                    resources: history_resources,
                }))
            }
            FHIRRequest::HistoryType(history_type_request) => {
                let history_resources = state
                    .repo
                    .history(
                        &context.ctx.tenant,
                        &context.ctx.project,
                        HistoryRequest::Type(&history_type_request),
                    )
                    .await?;

                Some(FHIRResponse::HistoryInstance(FHIRHistoryInstanceResponse {
                    resources: history_resources,
                }))
            }
            FHIRRequest::HistorySystem(history_system_request) => {
                let history_resources = state
                    .repo
                    .history(
                        &context.ctx.tenant,
                        &context.ctx.project,
                        HistoryRequest::System(&history_system_request),
                    )
                    .await?;

                Some(FHIRResponse::HistoryInstance(FHIRHistoryInstanceResponse {
                    resources: history_resources,
                }))
            }
            FHIRRequest::UpdateInstance(update_request) => {
                let resource = state
                    .repo
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
                        resource: FHIRRepository::update(
                            &state.repo,
                            &context.ctx.tenant,
                            &context.ctx.project,
                            &context.ctx.author,
                            &context.ctx.fhir_version,
                            &mut update_request.resource,
                            &update_request.id,
                        )
                        .await?,
                    }))
                } else {
                    Some(FHIRResponse::Create(FHIRCreateResponse {
                        resource: FHIRRepository::create(
                            &state.repo,
                            &context.ctx.tenant,
                            &context.ctx.project,
                            &context.ctx.author,
                            &context.ctx.fhir_version,
                            &mut update_request.resource,
                        )
                        .await?,
                    }))
                }
            }
            FHIRRequest::SearchType(search_type_request) => {
                let search_results = state
                    .search
                    .search(
                        &context.ctx.fhir_version,
                        &context.ctx.tenant,
                        &context.ctx.project,
                        SearchRequest::TypeSearch(search_type_request),
                    )
                    .await?;

                let resources = state
                    .repo
                    .read_by_version_ids(
                        &context.ctx.tenant,
                        &context.ctx.project,
                        search_results
                            .version_ids
                            .iter()
                            .map(|v| VersionIdRef::new(v))
                            .collect(),
                    )
                    .await?;

                Some(FHIRResponse::SearchType(FHIRSearchTypeResponse {
                    total: search_results.total,
                    resources,
                }))
            }
            FHIRRequest::ConditionalUpdate(request) => {
                let search_results = state
                    .search
                    .search(
                        &context.ctx.fhir_version,
                        &context.ctx.tenant,
                        &context.ctx.project,
                        SearchRequest::TypeSearch(&FHIRSearchTypeRequest {
                            resource_type: request.resource_type,
                            parameters: request
                                .parameters
                                .into_iter()
                                .filter(|p| match p {
                                    ParsedParameter::Resource(_) => true,
                                    _ => false,
                                })
                                .collect(),
                        }),
                    )
                    .await?;
                match search_results.version_ids.len() {
                    0 => {}
                    1 => {
                        let resource = search_results.into_iter().next().unwrap();
                        let found_id = resource
                            .get_field("id")
                            .unwrap()
                            .as_any()
                            .downcast_ref::<String>()
                            .unwrap();

                        self.update(ctx, resource_type, found_id, resource).await
                    }
                    _ => Err(OperationOutcomeError::error(
                        OperationOutcomeCodes::Conflict,
                        "Multiple resources found for conditional update.".to_string(),
                    )),
                }
            }
            _ => None,
        };

        let mut next_context = if let Some(next_) = next {
            next_(
                Arc::new(ClientState {
                    repo: state.repo.transaction().await.unwrap(),
                    search: state.search.clone(),
                }),
                context,
            )
            .await?
        } else {
            context
        };

        next_context.response = response;
        Ok(next_context)
    })
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

impl<Repo: Repository + Send + Sync + 'static, Search: SearchEngine + Send + Sync + 'static>
    FHIRServerClient<Repo, Search>
{
    pub fn new(repo: Repo, search: Search) -> Self {
        let middleware = Middleware::new(vec![Box::new(storage_middleware)]);
        FHIRServerClient {
            state: Arc::new(ClientState {
                repo,
                search: Arc::new(search),
            }),
            middleware,
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
        ctx: ServerCTX,
        resource_type: oxidized_fhir_model::r4::types::ResourceType,
        parameters: Vec<ParsedParameter>,
        resource: oxidized_fhir_model::r4::types::Resource,
    ) -> Result<oxidized_fhir_model::r4::types::Resource, OperationOutcomeError> {
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
