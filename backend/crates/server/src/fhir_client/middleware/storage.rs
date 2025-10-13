use crate::{
    fhir_client::{
        ClientState, FHIRServerClient, ServerCTX, StorageError,
        middleware::{
            ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput,
            ServerMiddlewareState,
        },
    },
    fhir_http::{self, HTTPRequest},
};
use axum::http::Method;
use oxidized_fhir_client::{
    FHIRClient,
    middleware::MiddlewareChain,
    request::{
        FHIRBatchResponse, FHIRCreateResponse, FHIRHistoryInstanceResponse, FHIRReadResponse,
        FHIRRequest, FHIRResponse, FHIRSearchSystemResponse, FHIRSearchTypeRequest,
        FHIRSearchTypeResponse, FHIRUpdateResponse, FHIRVersionReadResponse,
    },
    url::ParsedParameter,
};
use oxidized_fhir_model::r4::generated::{
    resources::{Bundle, BundleEntry, Resource},
    terminology::{BundleType, IssueType},
};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::{SearchEngine, SearchRequest};
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_reflect::MetaValue;
use oxidized_repository::{
    Repository,
    fhir::{FHIRRepository, HistoryRequest},
    types::{ResourceId, SupportedFHIRVersions, VersionIdRef},
};
use std::{str::FromStr, sync::Arc};

fn convert_bundle_entry(fhir_response: FHIRResponse) -> BundleEntry {
    BundleEntry {
        resource: match fhir_response {
            FHIRResponse::Create(res) => Some(Box::new(res.resource)),
            FHIRResponse::Read(res) => Some(Box::new(res.resource)),
            FHIRResponse::Update(res) => Some(Box::new(res.resource)),
            FHIRResponse::VersionRead(res) => Some(Box::new(res.resource)),
            FHIRResponse::DeleteInstance(_res) => None,
            FHIRResponse::DeleteType(_res) => None,
            FHIRResponse::DeleteSystem(_res) => None,
            FHIRResponse::HistoryInstance(res) => {
                let bundle = oxidized_fhir_client::axum::to_bundle(
                    BundleType::History(None),
                    None,
                    res.resources,
                );
                Some(Box::new(Resource::Bundle(bundle)))
            }
            FHIRResponse::HistoryType(res) => {
                let bundle = oxidized_fhir_client::axum::to_bundle(
                    BundleType::History(None),
                    None,
                    res.resources,
                );
                Some(Box::new(Resource::Bundle(bundle)))
            }
            FHIRResponse::HistorySystem(res) => {
                let bundle = oxidized_fhir_client::axum::to_bundle(
                    BundleType::History(None),
                    None,
                    res.resources,
                );
                Some(Box::new(Resource::Bundle(bundle)))
            }
            FHIRResponse::SearchSystem(res) => {
                let bundle = oxidized_fhir_client::axum::to_bundle(
                    BundleType::Searchset(None),
                    res.total,
                    res.resources,
                );
                Some(Box::new(Resource::Bundle(bundle)))
            }
            FHIRResponse::SearchType(res) => {
                let bundle = oxidized_fhir_client::axum::to_bundle(
                    BundleType::Searchset(None),
                    res.total,
                    res.resources,
                );
                Some(Box::new(Resource::Bundle(bundle)))
            }
            FHIRResponse::Patch(res) => Some(Box::new(res.resource)),

            FHIRResponse::Capabilities(res) => {
                Some(Box::new(Resource::CapabilityStatement(res.capabilities)))
            }

            FHIRResponse::InvokeInstance(res) => Some(Box::new(res.resource)),
            FHIRResponse::InvokeType(res) => Some(Box::new(res.resource)),
            FHIRResponse::InvokeSystem(res) => Some(Box::new(res.resource)),
            FHIRResponse::Batch(res) => Some(Box::new(Resource::Bundle(res.resource))),
            FHIRResponse::Transaction(res) => Some(Box::new(Resource::Bundle(res.resource))),
        },
        ..Default::default()
    }
}

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
        mut context: ServerMiddlewareContext,
        next: Option<Arc<ServerMiddlewareNext<Repo, Search, Terminology>>>,
    ) -> ServerMiddlewareOutput {
        Box::pin(async move {
            let response = match &mut context.request {
                FHIRRequest::Create(create_request) => {
                    Ok(Some(FHIRResponse::Create(FHIRCreateResponse {
                        resource: FHIRRepository::create(
                            state.repo.as_ref(),
                            &context.ctx.tenant,
                            &context.ctx.project,
                            &context.ctx.author,
                            &context.ctx.fhir_version,
                            &mut create_request.resource,
                        )
                        .await?,
                    })))
                }
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

                    Ok(Some(FHIRResponse::Read(FHIRReadResponse {
                        resource: resource,
                    })))
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
                        Ok(Some(FHIRResponse::VersionRead(FHIRVersionReadResponse {
                            resource: vread_resources.swap_remove(0),
                        })))
                    } else {
                        Ok(None)
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

                    Ok(Some(FHIRResponse::HistoryInstance(
                        FHIRHistoryInstanceResponse {
                            resources: history_resources,
                        },
                    )))
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

                    Ok(Some(FHIRResponse::HistoryInstance(
                        FHIRHistoryInstanceResponse {
                            resources: history_resources,
                        },
                    )))
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

                    Ok(Some(FHIRResponse::HistoryInstance(
                        FHIRHistoryInstanceResponse {
                            resources: history_resources,
                        },
                    )))
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

                        Ok(Some(FHIRResponse::Update(FHIRUpdateResponse {
                            resource: FHIRRepository::update(
                                state.repo.as_ref(),
                                &context.ctx.tenant,
                                &context.ctx.project,
                                &context.ctx.author,
                                &context.ctx.fhir_version,
                                &mut update_request.resource,
                                &update_request.id,
                            )
                            .await?,
                        })))
                    } else {
                        Ok(Some(FHIRResponse::Create(FHIRCreateResponse {
                            resource: FHIRRepository::create(
                                state.repo.as_ref(),
                                &context.ctx.tenant,
                                &context.ctx.project,
                                &context.ctx.author,
                                &context.ctx.fhir_version,
                                &mut update_request.resource,
                            )
                            .await?,
                        })))
                    }
                }
                FHIRRequest::SearchSystem(search_system_request) => {
                    let search_results = state
                        .search
                        .search(
                            &context.ctx.fhir_version,
                            &context.ctx.tenant,
                            &context.ctx.project,
                            SearchRequest::SystemSearch(search_system_request),
                            None,
                        )
                        .await?;

                    let resources = state
                        .repo
                        .read_by_version_ids(
                            &context.ctx.tenant,
                            &context.ctx.project,
                            search_results
                                .entries
                                .iter()
                                .map(|v| VersionIdRef::new(v.version_id.as_ref()))
                                .collect(),
                        )
                        .await?;

                    Ok(Some(FHIRResponse::SearchSystem(FHIRSearchSystemResponse {
                        total: search_results.total,
                        resources,
                    })))
                }
                FHIRRequest::SearchType(search_type_request) => {
                    let search_results = state
                        .search
                        .search(
                            &context.ctx.fhir_version,
                            &context.ctx.tenant,
                            &context.ctx.project,
                            SearchRequest::TypeSearch(search_type_request),
                            None,
                        )
                        .await?;

                    let resources = state
                        .repo
                        .read_by_version_ids(
                            &context.ctx.tenant,
                            &context.ctx.project,
                            search_results
                                .entries
                                .iter()
                                .map(|v| VersionIdRef::new(v.version_id.as_ref()))
                                .collect(),
                        )
                        .await?;

                    Ok(Some(FHIRResponse::SearchType(FHIRSearchTypeResponse {
                        total: search_results.total,
                        resources,
                    })))
                }
                FHIRRequest::ConditionalUpdate(update_request) => {
                    let search_results = state
                        .search
                        .search(
                            &context.ctx.fhir_version,
                            &context.ctx.tenant,
                            &context.ctx.project,
                            SearchRequest::TypeSearch(&FHIRSearchTypeRequest {
                                resource_type: update_request.resource_type.clone(),
                                parameters: update_request
                                    .parameters
                                    .clone()
                                    .into_iter()
                                    .filter(|p| match p {
                                        ParsedParameter::Resource(_) => true,
                                        _ => false,
                                    })
                                    .collect(),
                            }),
                            None,
                        )
                        .await?;
                    // No matches, no id provided:
                    //   The server creates the resource.
                    // No matches, id provided:
                    //   The server treats the interaction as an Update as Create interaction (or rejects it, if it does not support Update as Create)
                    match search_results.entries.len() {
                        0 => {
                            let id = update_request
                                .resource
                                .get_field("id")
                                .unwrap()
                                .as_any()
                                .downcast_ref::<String>()
                                .cloned();

                            // From R5 but Applying here on all versions to dissallow updating a Resource if it already exists
                            if let Some(id) = id {
                                let latest = state
                                    .repo
                                    .read_latest(
                                        &context.ctx.tenant,
                                        &context.ctx.project,
                                        &update_request.resource_type,
                                        &ResourceId::new(id.clone()),
                                    )
                                    .await?;

                                if latest.is_some() {
                                    return Err(OperationOutcomeError::error(
                                        IssueType::NotFound(None),
                                        "Resource exists but not found in conditional criteria."
                                            .to_string(),
                                    ));
                                }

                                Ok(Some(FHIRResponse::Update(FHIRUpdateResponse {
                                    resource: FHIRRepository::update(
                                        state.repo.as_ref(),
                                        &context.ctx.tenant,
                                        &context.ctx.project,
                                        &context.ctx.author,
                                        &context.ctx.fhir_version,
                                        &mut update_request.resource,
                                        &id,
                                    )
                                    .await?,
                                })))
                            } else {
                                Ok(Some(FHIRResponse::Create(FHIRCreateResponse {
                                    resource: FHIRRepository::create(
                                        state.repo.as_ref(),
                                        &context.ctx.tenant,
                                        &context.ctx.project,
                                        &context.ctx.author,
                                        &context.ctx.fhir_version,
                                        &mut update_request.resource,
                                    )
                                    .await?,
                                })))
                            }
                        }
                        1 => {
                            let search_result = search_results.entries.into_iter().next().unwrap();

                            if update_request.resource_type != search_result.resource_type {
                                return Err(OperationOutcomeError::error(
                                    IssueType::Conflict(None),
                                    "Resource type mismatch".to_string(),
                                ));
                            }

                            let resource_id_body = update_request
                                .resource
                                .get_field("id")
                                .ok_or_else(|| {
                                    OperationOutcomeError::error(
                                        IssueType::Invalid(None),
                                        "Missing resource ID".to_string(),
                                    )
                                })?
                                .as_any()
                                .downcast_ref::<String>();

                            // If body has resource Id verify it's the same as one in search result.
                            if resource_id_body.is_some()
                                && resource_id_body.as_ref().map(|s| s.as_str())
                                    != Some(search_result.id.as_ref())
                            {
                                return Err(OperationOutcomeError::error(
                                    IssueType::Conflict(None),
                                    "Resource ID mismatch".to_string(),
                                ));
                            }

                            Ok(Some(FHIRResponse::Update(FHIRUpdateResponse {
                                resource: FHIRRepository::update(
                                    state.repo.as_ref(),
                                    &context.ctx.tenant,
                                    &context.ctx.project,
                                    &context.ctx.author,
                                    &context.ctx.fhir_version,
                                    &mut update_request.resource,
                                    &search_result.id.as_ref(),
                                )
                                .await?,
                            })))
                        }
                        _ => Err(OperationOutcomeError::error(
                            IssueType::Conflict(None),
                            "Multiple resources found for conditional update.".to_string(),
                        )),
                    }
                }
                FHIRRequest::Batch(batch_request) => {
                    let mut bundle_entries = Some(Vec::new());
                    // Memswap so I can avoid cloning.
                    std::mem::swap(&mut batch_request.resource.entry, &mut bundle_entries);
                    let batch_client = FHIRServerClient::new(
                        state.repo.clone(),
                        state.search.clone(),
                        state.terminology.clone(),
                    );

                    let mut bundle_response = Bundle {
                        type_: Box::new(BundleType::BatchResponse(None)),
                        ..Default::default()
                    };
                    if let Some(bundle_entries) = bundle_entries {
                        let mut bundle_response_entries = Vec::with_capacity(bundle_entries.len());
                        for e in bundle_entries.into_iter() {
                            if let Some(request) = e.request.as_ref() {
                                let url = request
                                    .url
                                    .value
                                    .as_ref()
                                    .map(|s| s.as_str())
                                    .unwrap_or_default();

                                let (path, query) = url.split_once("?").unwrap_or((url, ""));
                                let request_method_string: Option<String> =
                                    request.method.as_ref().into();
                                let Ok(method) =
                                    Method::from_str(&request_method_string.unwrap_or_default())
                                else {
                                    return Err(OperationOutcomeError::error(
                                        IssueType::Invalid(None),
                                        "Invalid HTTP Method".to_string(),
                                    ));
                                };

                                let http_request = HTTPRequest::new(
                                    method,
                                    path.to_string(),
                                    if let Some(body) = e.resource {
                                        fhir_http::HTTPBody::Resource(*body)
                                    } else {
                                        fhir_http::HTTPBody::String("".to_string())
                                    },
                                    query.to_string(),
                                );

                                let Ok(fhir_request) = fhir_http::http_request_to_fhir_request(
                                    SupportedFHIRVersions::R4,
                                    http_request,
                                ) else {
                                    return Err(OperationOutcomeError::error(
                                        IssueType::Invalid(None),
                                        "Invalid Bundle entry".to_string(),
                                    ));
                                };

                                let fhir_response = batch_client
                                    .request(context.ctx.clone(), fhir_request)
                                    .await?;

                                bundle_response_entries.push(convert_bundle_entry(fhir_response));
                            }
                        }
                        bundle_response.entry = Some(bundle_response_entries);
                    }

                    Ok(Some(FHIRResponse::Batch(FHIRBatchResponse {
                        resource: bundle_response,
                    })))
                }
                _ => Ok(None),
            }?;

            let mut next_context = if let Some(next_) = next {
                next_(
                    Arc::new(ClientState {
                        repo: Arc::new(state.repo.transaction().await.unwrap()),
                        search: state.search.clone(),
                        terminology: state.terminology.clone(),
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
}
