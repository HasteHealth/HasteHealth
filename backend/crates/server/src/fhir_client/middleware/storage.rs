use crate::fhir_client::{
    FHIRServerClient, ServerCTX, ServerClientConfig, StorageError,
    batch_transaction_processing::{
        build_sorted_transaction_graph, process_batch_bundle, process_transaction_bundle,
    },
    middleware::{
        ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput,
        ServerMiddlewareState,
    },
};
use haste_fhir_client::{
    middleware::MiddlewareChain,
    request::{
        DeleteRequest, DeleteResponse, FHIRBatchResponse, FHIRCreateResponse,
        FHIRDeleteInstanceResponse, FHIRHistoryInstanceResponse, FHIRHistorySystemResponse,
        FHIRHistoryTypeResponse, FHIRPatchResponse, FHIRReadResponse, FHIRRequest, FHIRResponse,
        FHIRSearchSystemResponse, FHIRSearchTypeRequest, FHIRSearchTypeResponse,
        FHIRTransactionResponse, FHIRUpdateResponse, FHIRVersionReadResponse, HistoryRequest,
        HistoryResponse, SearchRequest, SearchResponse, UpdateRequest,
    },
    url::{ParsedParameter, ParsedParameters},
};
use haste_fhir_model::r4::generated::{
    resources::{Bundle, BundleEntry, Resource},
    terminology::{BundleType, IssueType},
    types::FHIRUnsignedInt,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::ResourceId;
use haste_reflect::MetaValue;
use haste_repository::{Repository, fhir::FHIRRepository};
use std::{
    io::{BufWriter, Write},
    sync::Arc,
};

pub struct Middleware {}
impl Middleware {
    pub fn new() -> Self {
        Middleware {}
    }
}

pub fn to_bundle(bundle_type: BundleType, total: Option<i64>, resources: Vec<Resource>) -> Bundle {
    Bundle {
        id: None,
        meta: None,
        total: total.map(|t| {
            Box::new(FHIRUnsignedInt {
                value: Some(t as u64),
                ..Default::default()
            })
        }),
        type_: Box::new(bundle_type),
        entry: Some(
            resources
                .into_iter()
                .map(|r| BundleEntry {
                    resource: Some(Box::new(r)),
                    ..Default::default()
                })
                .collect(),
        ),
        ..Default::default()
    }
}

impl<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>
    MiddlewareChain<
        ServerMiddlewareState<Repo, Search, Terminology>,
        Arc<ServerCTX<Repo, Search, Terminology>>,
        FHIRRequest,
        FHIRResponse,
        OperationOutcomeError,
    > for Middleware
{
    fn call(
        &self,
        state: ServerMiddlewareState<Repo, Search, Terminology>,
        mut context: ServerMiddlewareContext<Repo, Search, Terminology>,
        next: Option<Arc<ServerMiddlewareNext<Repo, Search, Terminology>>>,
    ) -> ServerMiddlewareOutput<Repo, Search, Terminology> {
        Box::pin(async move {
            let response = match &mut context.request {
                FHIRRequest::Create(create_request) => {
                    Ok(Some(FHIRResponse::Create(FHIRCreateResponse {
                        resource: FHIRRepository::create(
                            state.repo.as_ref(),
                            &context.ctx.tenant,
                            &context.ctx.project,
                            &context.ctx.user,
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
                        .await?;

                    Ok(Some(FHIRResponse::Read(FHIRReadResponse {
                        resource: resource,
                    })))
                }
                FHIRRequest::Delete(DeleteRequest::Instance(delete_request)) => {
                    let current_resource = FHIRRepository::read_latest(
                        state.repo.as_ref(),
                        &context.ctx.tenant,
                        &context.ctx.project,
                        &delete_request.resource_type,
                        &ResourceId::new(delete_request.id.to_string()),
                    )
                    .await?;
                    if let Some(mut resource) = current_resource {
                        Ok(Some(FHIRResponse::Delete(DeleteResponse::Instance(
                            FHIRDeleteInstanceResponse {
                                resource: FHIRRepository::delete(
                                    state.repo.as_ref(),
                                    &context.ctx.tenant,
                                    &context.ctx.project,
                                    &context.ctx.user,
                                    &context.ctx.fhir_version,
                                    &mut resource,
                                    &delete_request.id,
                                )
                                .await?,
                            },
                        ))))
                    } else {
                        Err(OperationOutcomeError::error(
                            IssueType::NotFound(None),
                            format!("Resource with id '{}' not found", delete_request.id),
                        ))
                    }
                }

                FHIRRequest::VersionRead(vread_request) => {
                    let mut vread_resources = state
                        .repo
                        .read_by_version_ids(
                            &context.ctx.tenant,
                            &context.ctx.project,
                            &[&vread_request.version_id],
                            haste_repository::fhir::CachePolicy::Cache,
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

                FHIRRequest::History(history_request) => match history_request {
                    HistoryRequest::Instance(_) => {
                        let history_resources = state
                            .repo
                            .history(&context.ctx.tenant, &context.ctx.project, &history_request)
                            .await?;

                        Ok(Some(FHIRResponse::History(HistoryResponse::Instance(
                            FHIRHistoryInstanceResponse {
                                bundle: to_bundle(
                                    BundleType::History(None),
                                    None,
                                    history_resources,
                                ),
                            },
                        ))))
                    }
                    HistoryRequest::Type(_) => {
                        let history_resources = state
                            .repo
                            .history(&context.ctx.tenant, &context.ctx.project, &history_request)
                            .await?;

                        Ok(Some(FHIRResponse::History(HistoryResponse::Type(
                            FHIRHistoryTypeResponse {
                                bundle: to_bundle(
                                    BundleType::History(None),
                                    None,
                                    history_resources,
                                ),
                            },
                        ))))
                    }
                    HistoryRequest::System(_) => {
                        let history_resources: Vec<Resource> = state
                            .repo
                            .history(&context.ctx.tenant, &context.ctx.project, &history_request)
                            .await?;

                        Ok(Some(FHIRResponse::History(HistoryResponse::System(
                            FHIRHistorySystemResponse {
                                bundle: to_bundle(
                                    BundleType::History(None),
                                    None,
                                    history_resources,
                                ),
                            },
                        ))))
                    }
                },
                FHIRRequest::Update(update_request) => match update_request {
                    UpdateRequest::Instance(update_request) => {
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
                                    &context.ctx.user,
                                    &context.ctx.fhir_version,
                                    &mut update_request.resource,
                                    &update_request.id,
                                )
                                .await?,
                            })))
                        } else {
                            // Create the resource if it does not exist. With the given id.
                            Ok(Some(FHIRResponse::Create(FHIRCreateResponse {
                                resource: FHIRRepository::update(
                                    state.repo.as_ref(),
                                    &context.ctx.tenant,
                                    &context.ctx.project,
                                    &context.ctx.user,
                                    &context.ctx.fhir_version,
                                    &mut update_request.resource,
                                    &update_request.id,
                                )
                                .await?,
                            })))
                        }
                    }
                    UpdateRequest::Conditional(update_request) => {
                        let search_results = state
                            .search
                            .search(
                                &context.ctx.fhir_version,
                                &context.ctx.tenant,
                                &context.ctx.project,
                                &SearchRequest::Type(FHIRSearchTypeRequest {
                                    resource_type: update_request.resource_type.clone(),
                                    parameters: ParsedParameters::new(
                                        update_request
                                            .parameters
                                            .parameters()
                                            .clone()
                                            .into_iter()
                                            .filter(|p| match p {
                                                ParsedParameter::Resource(_) => true,
                                                _ => false,
                                            })
                                            .collect(),
                                    ),
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
                                            &context.ctx.user,
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
                                            &context.ctx.user,
                                            &context.ctx.fhir_version,
                                            &mut update_request.resource,
                                        )
                                        .await?,
                                    })))
                                }
                            }
                            1 => {
                                let search_result =
                                    search_results.entries.into_iter().next().unwrap();

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
                                        &context.ctx.user,
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
                },

                FHIRRequest::Search(search_request) => match search_request {
                    SearchRequest::Type(_) => {
                        let search_results = state
                            .search
                            .search(
                                &context.ctx.fhir_version,
                                &context.ctx.tenant,
                                &context.ctx.project,
                                &search_request,
                                None,
                            )
                            .await?;
                        let version_ids = search_results
                            .entries
                            .iter()
                            .map(|v| &v.version_id)
                            .collect::<Vec<_>>();

                        let resources = state
                            .repo
                            .read_by_version_ids(
                                &context.ctx.tenant,
                                &context.ctx.project,
                                version_ids.as_slice(),
                                haste_repository::fhir::CachePolicy::NoCache,
                            )
                            .await?;

                        Ok(Some(FHIRResponse::Search(SearchResponse::Type(
                            FHIRSearchTypeResponse {
                                bundle: to_bundle(
                                    BundleType::Searchset(None),
                                    search_results.total,
                                    resources,
                                ),
                            },
                        ))))
                    }
                    SearchRequest::System(_) => {
                        let search_results = state
                            .search
                            .search(
                                &context.ctx.fhir_version,
                                &context.ctx.tenant,
                                &context.ctx.project,
                                &search_request,
                                None,
                            )
                            .await?;
                        let version_ids = search_results
                            .entries
                            .iter()
                            .map(|v| &v.version_id)
                            .collect::<Vec<_>>();

                        let resources = state
                            .repo
                            .read_by_version_ids(
                                &context.ctx.tenant,
                                &context.ctx.project,
                                version_ids.as_slice(),
                                haste_repository::fhir::CachePolicy::NoCache,
                            )
                            .await?;

                        Ok(Some(FHIRResponse::Search(SearchResponse::System(
                            FHIRSearchSystemResponse {
                                bundle: to_bundle(
                                    BundleType::Searchset(None),
                                    search_results.total,
                                    resources,
                                ),
                            },
                        ))))
                    }
                },

                FHIRRequest::Transaction(transaction_request) => {
                    let mut transaction_entries: Option<Vec<BundleEntry>> = None;
                    // Memswap so I can avoid cloning.
                    std::mem::swap(
                        &mut transaction_request.resource.entry,
                        &mut transaction_entries,
                    );

                    // Run sort before creating transaction.
                    // So that transaction is only used for the direct submission of the sorted entries.
                    // We want to limit time within a transaction as much as possible.
                    let sorted_transaction =
                        build_sorted_transaction_graph(transaction_entries.unwrap_or_default())?;

                    let transaction_repo = Arc::new(state.repo.transaction(true).await?);

                    let bundle_response: Result<Bundle, OperationOutcomeError> = {
                        let transaction_client = FHIRServerClient::new(ServerClientConfig::new(
                            transaction_repo.clone(),
                            state.search.clone(),
                            state.terminology.clone(),
                            state.config.clone(),
                        ));

                        Ok(process_transaction_bundle(
                            &transaction_client,
                            context.ctx.clone(),
                            sorted_transaction,
                        )
                        .await?)
                    };

                    let repo = Arc::try_unwrap(transaction_repo).map_err(|_e| {
                        OperationOutcomeError::fatal(
                            IssueType::Exception(None),
                            "Failed to unwrap transaction client".to_string(),
                        )
                    })?;

                    if let Ok(transaction_bundle) = bundle_response {
                        repo.commit().await?;
                        Ok(Some(FHIRResponse::Transaction(FHIRTransactionResponse {
                            resource: transaction_bundle,
                        })))
                    } else if let Err(operation_error) = bundle_response {
                        tracing::info!("Rolling back transaction due to error");
                        repo.rollback().await?;

                        Err(operation_error)
                    } else {
                        Err(OperationOutcomeError::fatal(
                            IssueType::Exception(None),
                            "Unexpected transaction error".to_string(),
                        ))
                    }
                }
                FHIRRequest::Batch(batch_request) => {
                    let mut batch_entries: Option<Vec<BundleEntry>> = None;
                    // Memswap so I can avoid cloning.
                    std::mem::swap(&mut batch_request.resource.entry, &mut batch_entries);
                    let batch_client = FHIRServerClient::new(ServerClientConfig::new(
                        state.repo.clone(),
                        state.search.clone(),
                        state.terminology.clone(),
                        state.config.clone(),
                    ));

                    Ok(Some(FHIRResponse::Batch(FHIRBatchResponse {
                        resource: process_batch_bundle(
                            &batch_client,
                            context.ctx.clone(),
                            batch_entries.unwrap_or_else(Vec::new),
                        )
                        .await?,
                    })))
                }
                FHIRRequest::Patch(fhir_patch_request) => {
                    let Some(resource) = state
                        .repo
                        .read_latest(
                            &context.ctx.tenant,
                            &context.ctx.project,
                            &fhir_patch_request.resource_type,
                            &ResourceId::new(fhir_patch_request.id.to_string()),
                        )
                        .await?
                    else {
                        return Err(OperationOutcomeError::error(
                            IssueType::NotFound(None),
                            format!("Resource with id '{}' not found", fhir_patch_request.id),
                        ));
                    };

                    let mut writer = BufWriter::new(Vec::new());
                    haste_fhir_serialization_json::to_writer(&mut writer, &resource).map_err(
                        |e| {
                            OperationOutcomeError::fatal(
                                IssueType::Exception(None),
                                "Failed to serialize resource for patching: ".to_string()
                                    + &e.to_string(),
                            )
                        },
                    )?;
                    writer.flush().map_err(|e| {
                        OperationOutcomeError::fatal(
                            IssueType::Exception(None),
                            "Failed to flush buffer: ".to_string() + &e.to_string(),
                        )
                    })?;

                    let content: Vec<u8> = writer.into_inner().map_err(|e| {
                        OperationOutcomeError::fatal(
                            IssueType::Exception(None),
                            "Failed to retrieve buffer content: ".to_string() + &e.to_string(),
                        )
                    })?;

                    let mut json: serde_json::Value = serde_json::from_reader(content.as_slice())
                        .map_err(|e| {
                        OperationOutcomeError::fatal(
                            IssueType::Exception(None),
                            "Failed to deserialize JSON for patching: ".to_string()
                                + &e.to_string(),
                        )
                    })?;

                    json_patch::patch(&mut json, &fhir_patch_request.patch).map_err(|e| {
                        OperationOutcomeError::fatal(
                            IssueType::Exception(None),
                            format!("Failed to apply JSON patch: '{}'", e.to_string()),
                        )
                    })?;

                    let mut patched_resource =
                        haste_fhir_serialization_json::from_serde_value::<Resource>(&json)
                            .map_err(|e| {
                                OperationOutcomeError::fatal(
                                    IssueType::Exception(None),
                                    format!("Failed to deserialize patched resource '{}'.", e),
                                )
                            })?;

                    if std::mem::discriminant(&resource)
                        != std::mem::discriminant(&patched_resource)
                    {
                        return Err(OperationOutcomeError::error(
                            IssueType::Conflict(None),
                            "Resource type mismatch after patching".to_string(),
                        ));
                    }

                    let patched_id = patched_resource
                        .get_field("id")
                        .ok_or_else(|| {
                            OperationOutcomeError::error(
                                IssueType::Invalid(None),
                                "Missing resource ID".to_string(),
                            )
                        })?
                        .as_any()
                        .downcast_ref::<String>()
                        .cloned()
                        .ok_or_else(|| {
                            OperationOutcomeError::error(
                                IssueType::Invalid(None),
                                "Invalid resource ID type".to_string(),
                            )
                        })?;

                    if fhir_patch_request.id != patched_id {
                        return Err(OperationOutcomeError::error(
                            IssueType::Conflict(None),
                            "Resource ID mismatch after patching".to_string(),
                        ));
                    }

                    Ok(Some(FHIRResponse::Patch(FHIRPatchResponse {
                        resource: FHIRRepository::update(
                            state.repo.as_ref(),
                            &context.ctx.tenant,
                            &context.ctx.project,
                            &context.ctx.user,
                            &context.ctx.fhir_version,
                            &mut patched_resource,
                            &fhir_patch_request.id,
                        )
                        .await?,
                    })))
                }
                FHIRRequest::Delete(DeleteRequest::Type(_))
                | FHIRRequest::Delete(DeleteRequest::System(_))
                | FHIRRequest::Capabilities
                | FHIRRequest::Invocation(_) => Err(OperationOutcomeError::error(
                    IssueType::NotSupported(None),
                    "Unsupported FHIR operation".to_string(),
                )),
            }?;

            let mut next_context = if let Some(next_) = next {
                next_(state.clone(), context).await?
            } else {
                context
            };

            next_context.response = response;
            Ok(next_context)
        })
    }
}
