use crate::{
    fhir_client::{
        ClientState, FHIRServerClient, ServerCTX, ServerClientConfig, StorageError,
        batch_transaction_processing::{
            build_sorted_transaction_graph, process_batch_bundle, process_transaction_bundle,
        },
        compartment::process_compartment_request,
        middleware::{Backend, Handler, Output, auth_z::patient_scope, search_includes},
    },
    route_path::{api_fhir_root_url, append_path_segments},
};
use haste_fhir_client::{
    FHIRClient,
    request::{
        DeleteRequest, DeleteResponse, FHIRBatchResponse, FHIRCreateResponse,
        FHIRDeleteInstanceResponse, FHIRDeleteSystemResponse, FHIRDeleteTypeResponse,
        FHIRHistoryInstanceResponse, FHIRHistorySystemResponse, FHIRHistoryTypeResponse,
        FHIRPatchRequest, FHIRPatchResponse, FHIRReadResponse, FHIRRequest, FHIRResponse,
        FHIRSearchSystemRequest, FHIRSearchSystemResponse, FHIRSearchTypeRequest,
        FHIRSearchTypeResponse, FHIRTransactionResponse, FHIRUpdateResponse,
        FHIRVersionReadResponse, HistoryRequest, HistoryResponse, SearchRequest, SearchResponse,
        UpdateRequest,
    },
    url::{ParsedParameter, ParsedParameters},
};
use haste_fhir_model::r4::generated::{
    resources::{Bundle, BundleEntry, BundleEntryRequest, BundleEntrySearch, Resource},
    terminology::{BoundCode, BundleType, HttpVerb, IssueType, SearchEntryMode},
    types::{FHIRUnsignedInt, FHIRUri},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::{SearchEngine, SearchEntry, SearchOptions};
use haste_jwt::{ProjectId, ResourceId, TenantId};
use haste_reflect::MetaValue;
use haste_repository::{Repository, fhir::FHIRRepository};
use std::sync::Arc;
use url::Url;

/// Applies a PATCH to the latest version of a resource and returns the patched
/// resource, without saving it. The patch may not change the resource's type
/// or `id`.
///
/// Storage saves the result as an update. Routes whose middleware must see the
/// change as an update (tenant and project auth resources) use it through
/// [`super::patch_as_update`].
pub async fn apply_patch<Repo: Repository>(
    repo: &Repo,
    tenant: &TenantId,
    project: &ProjectId,
    fhir_patch_request: &FHIRPatchRequest,
) -> Result<Resource, OperationOutcomeError> {
    let Some(resource) = repo
        .read_latest(
            tenant,
            project,
            &fhir_patch_request.resource_type,
            &ResourceId::new(fhir_patch_request.id.to_string()),
        )
        .await?
    else {
        return Err(OperationOutcomeError::error(
            IssueType::not_found(),
            format!("Resource with id '{}' not found", fhir_patch_request.id),
        ));
    };

    let mut json: serde_json::Value = serde_json::to_value(&resource).map_err(|e| {
        OperationOutcomeError::fatal(
            IssueType::exception(),
            "Failed to deserialize JSON for patching: ".to_string() + (e.to_string().as_str()),
        )
    })?;

    json_patch::patch(&mut json, &fhir_patch_request.patch).map_err(|e| {
        OperationOutcomeError::fatal(
            IssueType::invalid(),
            format!("Failed to apply JSON patch: '{}'", e),
        )
    })?;

    let patched_resource = serde_json::from_value::<Resource>(json).map_err(|e| {
        OperationOutcomeError::fatal(
            IssueType::invalid(),
            format!("Failed to deserialize patched resource '{}'.", e),
        )
    })?;

    if std::mem::discriminant(&resource) != std::mem::discriminant(&patched_resource) {
        return Err(OperationOutcomeError::error(
            IssueType::conflict(),
            "Resource type mismatch after patching".to_string(),
        ));
    }

    let patched_id = patched_resource
        .get_field("id")
        .ok_or_else(|| {
            OperationOutcomeError::error(IssueType::invalid(), "Missing resource ID".to_string())
        })?
        .as_any()
        .downcast_ref::<String>()
        .cloned()
        .ok_or_else(|| {
            OperationOutcomeError::error(
                IssueType::invalid(),
                "Invalid resource ID type".to_string(),
            )
        })?;

    if fhir_patch_request.id != patched_id {
        return Err(OperationOutcomeError::error(
            IssueType::conflict(),
            "Resource ID mismatch after patching".to_string(),
        ));
    }

    Ok(patched_resource)
}

/// Ends a lane: reads and writes the repository and search index.
pub struct Storage;

pub fn to_bundle_entry(
    fhir_api_url: &Url,
    resource: Resource,
    request_method: Option<String>,
) -> BundleEntry {
    let resource_type = resource.resource_type();
    let id = resource.id().as_ref().map(|s| s.as_str()).unwrap_or("");

    let mut entry = BundleEntry::default();

    if let Some(request_method) = request_method {
        let http_verb = BoundCode::<HttpVerb>::new(&request_method).unwrap_or(HttpVerb::post());
        entry.request = Some(BundleEntryRequest {
            url: Box::new(FHIRUri {
                value: match &http_verb {
                    h if h == &HttpVerb::post() => Some(resource_type.as_ref().to_string()),
                    h if h == &HttpVerb::delete()
                        || h == &HttpVerb::put()
                        || h == &HttpVerb::patch() =>
                    {
                        Some(format!("{}/{}", resource_type.as_ref(), id))
                    }
                    _ => None,
                },
                ..Default::default()
            }),
            method: http_verb,
            ..Default::default()
        });
    }

    entry.fullUrl = append_path_segments(fhir_api_url, [resource_type.as_ref(), id]).map(|url| {
        Box::new(FHIRUri {
            value: Some(url.to_string()),
            ..Default::default()
        })
    });

    entry.resource = Some(Box::new(resource));

    entry
}

/// The options a search runs with. A caller confined to a patient's
/// compartment gets the filter that keeps every search there.
fn search_options(
    compartment: Option<&str>,
    request: &SearchRequest,
    count_limit: Option<u64>,
) -> Option<SearchOptions> {
    let resource_type = match request {
        SearchRequest::Type(search) => Some(&search.resource_type),
        SearchRequest::System(_) => None,
    };
    Some(SearchOptions {
        count_limit,
        any_of: compartment
            .and_then(|patient_id| patient_scope::search_filter(patient_id, resource_type)),
    })
}

/// The parameters of a search request, whichever level it is at.
fn search_request_parameters(request: &SearchRequest) -> &ParsedParameters {
    match request {
        SearchRequest::Type(r) => &r.parameters,
        SearchRequest::System(r) => &r.parameters,
    }
}

/// One page's entries: the matches, then whatever `_include` added.
///
/// `search.mode` is set on every entry. FHIR requires it to distinguish a
/// match from an include, and a client that renders an included Patient as a
/// search hit is showing the user a result they did not ask for.
fn search_entries(
    fhir_api_root: &Url,
    matches: Vec<Resource>,
    included: Vec<Resource>,
) -> Vec<BundleEntry> {
    let entry = |resource, mode: BoundCode<SearchEntryMode>| {
        let mut entry = to_bundle_entry(fhir_api_root, resource, None);
        entry.search = Some(BundleEntrySearch {
            mode: Some(mode),
            ..Default::default()
        });
        entry
    };

    matches
        .into_iter()
        .map(|resource| entry(resource, SearchEntryMode::match_()))
        .chain(
            included
                .into_iter()
                .map(|resource| entry(resource, SearchEntryMode::include())),
        )
        .collect()
}

pub fn to_bundle(
    bundle_type: BoundCode<BundleType>,
    total: Option<i64>,
    entries: Vec<BundleEntry>,
) -> Bundle {
    Bundle {
        id: None,
        meta: None,
        total: total.map(|t| {
            Box::new(FHIRUnsignedInt {
                value: Some(t as u64),
                ..Default::default()
            })
        }),
        type_: bundle_type,
        entry: Some(entries),
        ..Default::default()
    }
}

/// Deletes every match of `search`, each as its own instance delete through
/// the client so it gets its lane's checks and sync hooks. The deletes commit
/// together: in the lane's transaction when it opened one, else in one here.
async fn delete_matches<B: Backend>(
    state: &ClientState<B>,
    ctx: &ServerCTX<B>,
    search: SearchRequest,
) -> Result<(), OperationOutcomeError> {
    let delete_limit = state.config.fhir.delete_limit;
    let found = state
        .search
        .search(
            &ctx.fhir_version,
            &ctx.tenant,
            &ctx.project,
            &search,
            search_options(ctx.compartment.as_deref(), &search, Some(delete_limit + 1)),
        )
        .await?;
    if found.entries.len() as u64 > delete_limit {
        return Err(OperationOutcomeError::error(
            IssueType::invalid(),
            format!("Too many resources to delete at once. Limit to '{delete_limit}'."),
        ));
    }

    if state.repo.in_transaction() {
        return delete_each(state, state.repo.clone(), ctx, found.entries).await;
    }

    let repo = Arc::new(state.repo.transaction(true).await?);
    let result = delete_each(state, repo.clone(), ctx, found.entries).await;
    let repo = Arc::try_unwrap(repo).map_err(|_| {
        OperationOutcomeError::fatal(
            IssueType::exception(),
            "Failed to unwrap transaction client".to_string(),
        )
    })?;
    match result {
        Ok(()) => repo.commit().await,
        Err(error) => {
            repo.rollback().await?;
            Err(error)
        }
    }
}

/// One instance delete per entry, through a client on `repo`.
async fn delete_each<B: Backend>(
    state: &ClientState<B>,
    repo: Arc<B::Repo>,
    ctx: &ServerCTX<B>,
    entries: Vec<SearchEntry>,
) -> Result<(), OperationOutcomeError> {
    let client = FHIRServerClient::new(
        ServerClientConfig::new(
            repo,
            state.search.clone(),
            state.terminology.clone(),
            state.config.clone(),
            state.deno_pool.clone(),
        )
        .with_audit_repo(state.audit_repo.clone()),
    );
    let ctx = Arc::new(ctx.swap_client(Arc::new(client)));
    for entry in entries {
        ctx.client
            .delete_instance(
                ctx.clone(),
                entry.resource_type,
                entry.id.as_ref().to_string(),
            )
            .await?;
    }
    Ok(())
}

impl<B: Backend> Handler<B> for Storage {
    #[tracing::instrument(name = "storage_middleware", skip(self, state))]
    fn handle<'a>(
        &'a self,
        state: &'a ClientState<B>,
        ctx: Arc<ServerCTX<B>>,
        request: FHIRRequest,
    ) -> Output<'a> {
        Box::pin(async move {
            match request {
                FHIRRequest::Create(create_request) => {
                    Ok(FHIRResponse::Create(FHIRCreateResponse {
                        resource: FHIRRepository::create(
                            state.repo.as_ref(),
                            &ctx.tenant,
                            &ctx.project,
                            &ctx.user.claims,
                            &ctx.fhir_version,
                            create_request.resource,
                        )
                        .await?,
                    }))
                }
                FHIRRequest::Read(read_request) => {
                    let resource = state
                        .repo
                        .read_latest(
                            &ctx.tenant,
                            &ctx.project,
                            &read_request.resource_type,
                            &ResourceId::new(read_request.id),
                        )
                        .await?;

                    Ok(FHIRResponse::Read(FHIRReadResponse { resource }))
                }
                FHIRRequest::Delete(DeleteRequest::Instance(delete_request)) => {
                    let current_resource = FHIRRepository::read_latest(
                        state.repo.as_ref(),
                        &ctx.tenant,
                        &ctx.project,
                        &delete_request.resource_type,
                        &ResourceId::new(delete_request.id.clone()),
                    )
                    .await?;
                    let Some(resource) = current_resource else {
                        return Err(OperationOutcomeError::error(
                            IssueType::not_found(),
                            format!("Resource with id '{}' not found", delete_request.id),
                        ));
                    };

                    Ok(FHIRResponse::Delete(DeleteResponse::Instance(Box::new(
                        FHIRDeleteInstanceResponse {
                            resource: FHIRRepository::delete(
                                state.repo.as_ref(),
                                &ctx.tenant,
                                &ctx.project,
                                &ctx.user.claims,
                                &ctx.fhir_version,
                                resource,
                                &delete_request.id,
                            )
                            .await?,
                        },
                    ))))
                }
                FHIRRequest::Delete(DeleteRequest::Type(delete_request)) => {
                    delete_matches(
                        state,
                        &ctx,
                        SearchRequest::Type(FHIRSearchTypeRequest {
                            resource_type: delete_request.resource_type,
                            parameters: delete_request.parameters,
                        }),
                    )
                    .await?;

                    Ok(FHIRResponse::Delete(DeleteResponse::Type(
                        FHIRDeleteTypeResponse {},
                    )))
                }
                FHIRRequest::Delete(DeleteRequest::System(delete_request)) => {
                    delete_matches(
                        state,
                        &ctx,
                        SearchRequest::System(FHIRSearchSystemRequest {
                            parameters: delete_request.parameters,
                        }),
                    )
                    .await?;

                    Ok(FHIRResponse::Delete(DeleteResponse::System(
                        FHIRDeleteSystemResponse {},
                    )))
                }
                FHIRRequest::VersionRead(vread_request) => {
                    let mut vread_resources = state
                        .repo
                        .read_by_version_ids(
                            &ctx.tenant,
                            &ctx.project,
                            &[&vread_request.version_id],
                            haste_repository::fhir::CachePolicy::Cache,
                        )
                        .await?;

                    if vread_resources.is_empty() {
                        return Err(OperationOutcomeError::error(
                            IssueType::not_found(),
                            format!(
                                "Version '{}' of resource '{}/{}' not found",
                                vread_request.version_id.as_ref(),
                                vread_request.resource_type.as_ref(),
                                vread_request.id
                            ),
                        ));
                    }
                    Ok(FHIRResponse::VersionRead(FHIRVersionReadResponse {
                        resource: vread_resources.swap_remove(0),
                    }))
                }
                FHIRRequest::History(history_request) => {
                    let history_resources = state
                        .repo
                        .history(&ctx.tenant, &ctx.project, &history_request)
                        .await?;

                    let fhir_api_root =
                        api_fhir_root_url(&state.config.api_uri, &ctx.tenant, &ctx.project)?;

                    let bundle = to_bundle(
                        BundleType::history(),
                        None,
                        history_resources
                            .into_iter()
                            .map(|r| {
                                to_bundle_entry(&fhir_api_root, r.resource, Some(r.request_method))
                            })
                            .collect(),
                    );

                    Ok(FHIRResponse::History(match history_request {
                        HistoryRequest::Instance(_) => {
                            HistoryResponse::Instance(FHIRHistoryInstanceResponse { bundle })
                        }
                        HistoryRequest::Type(_) => {
                            HistoryResponse::Type(FHIRHistoryTypeResponse { bundle })
                        }
                        HistoryRequest::System(_) => {
                            HistoryResponse::System(FHIRHistorySystemResponse { bundle })
                        }
                    }))
                }
                FHIRRequest::Update(UpdateRequest::Instance(update_request)) => {
                    let current = state
                        .repo
                        .read_latest(
                            &ctx.tenant,
                            &ctx.project,
                            &update_request.resource_type,
                            &ResourceId::new(update_request.id.clone()),
                        )
                        .await?;

                    if let Some(current) = &current
                        && std::mem::discriminant(current)
                            != std::mem::discriminant(&update_request.resource)
                    {
                        return Err(StorageError::InvalidType.into());
                    }

                    let resource = FHIRRepository::update(
                        state.repo.as_ref(),
                        &ctx.tenant,
                        &ctx.project,
                        &ctx.user.claims,
                        &ctx.fhir_version,
                        update_request.resource,
                        &update_request.id,
                    )
                    .await?;

                    // Update as create: a resource that did not exist is
                    // created with the given id.
                    Ok(match current {
                        Some(_) => FHIRResponse::Update(FHIRUpdateResponse { resource }),
                        None => FHIRResponse::Create(FHIRCreateResponse { resource }),
                    })
                }
                FHIRRequest::Update(UpdateRequest::Conditional(update_request)) => {
                    let update_parameters = update_request
                        .parameters
                        .parameters()
                        .iter()
                        .filter(|p| matches!(p, ParsedParameter::Resource(_)))
                        .cloned()
                        .collect();

                    let update_search = SearchRequest::Type(FHIRSearchTypeRequest {
                        resource_type: update_request.resource_type.clone(),
                        parameters: ParsedParameters::new(update_parameters),
                    });
                    let search_results = state
                        .search
                        .search(
                            &ctx.fhir_version,
                            &ctx.tenant,
                            &ctx.project,
                            &update_search,
                            search_options(ctx.compartment.as_deref(), &update_search, None),
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
                                .and_then(|id| id.as_any().downcast_ref::<String>())
                                .cloned();

                            // From R5 but Applying here on all versions to dissallow updating a Resource if it already exists
                            if let Some(id) = id {
                                let latest = state
                                    .repo
                                    .read_latest(
                                        &ctx.tenant,
                                        &ctx.project,
                                        &update_request.resource_type,
                                        &ResourceId::new(id.clone()),
                                    )
                                    .await?;

                                if latest.is_some() {
                                    return Err(OperationOutcomeError::error(
                                        IssueType::conflict(),
                                        "Resource exists but not found in conditional criteria."
                                            .to_string(),
                                    ));
                                }

                                Ok(FHIRResponse::Update(FHIRUpdateResponse {
                                    resource: FHIRRepository::update(
                                        state.repo.as_ref(),
                                        &ctx.tenant,
                                        &ctx.project,
                                        &ctx.user.claims,
                                        &ctx.fhir_version,
                                        update_request.resource,
                                        &id,
                                    )
                                    .await?,
                                }))
                            } else {
                                Ok(FHIRResponse::Create(FHIRCreateResponse {
                                    resource: FHIRRepository::create(
                                        state.repo.as_ref(),
                                        &ctx.tenant,
                                        &ctx.project,
                                        &ctx.user.claims,
                                        &ctx.fhir_version,
                                        update_request.resource,
                                    )
                                    .await?,
                                }))
                            }
                        }
                        1 => {
                            let search_result = search_results.entries.into_iter().next().unwrap();

                            if update_request.resource_type != search_result.resource_type {
                                return Err(OperationOutcomeError::error(
                                    IssueType::conflict(),
                                    "Resource type mismatch".to_string(),
                                ));
                            }

                            let resource_id_body = update_request
                                .resource
                                .get_field("id")
                                .and_then(|id| id.as_any().downcast_ref::<String>());

                            // If body has resource Id verify it's the same as one in search result.
                            if resource_id_body.is_some()
                                && resource_id_body.map(String::as_str)
                                    != Some(search_result.id.as_ref())
                            {
                                return Err(OperationOutcomeError::error(
                                    IssueType::conflict(),
                                    "Resource ID mismatch".to_string(),
                                ));
                            }

                            Ok(FHIRResponse::Update(FHIRUpdateResponse {
                                resource: FHIRRepository::update(
                                    state.repo.as_ref(),
                                    &ctx.tenant,
                                    &ctx.project,
                                    &ctx.user.claims,
                                    &ctx.fhir_version,
                                    update_request.resource,
                                    search_result.id.as_ref(),
                                )
                                .await?,
                            }))
                        }
                        _ => Err(OperationOutcomeError::error(
                            IssueType::conflict(),
                            "Multiple resources found for conditional update.".to_string(),
                        )),
                    }
                }
                FHIRRequest::Search(search_request) => {
                    let fhir_api_root =
                        api_fhir_root_url(&state.config.api_uri, &ctx.tenant, &ctx.project)?;

                    let search_results = state
                        .search
                        .search(
                            &ctx.fhir_version,
                            &ctx.tenant,
                            &ctx.project,
                            &search_request,
                            search_options(ctx.compartment.as_deref(), &search_request, None),
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
                            &ctx.tenant,
                            &ctx.project,
                            version_ids.as_slice(),
                            haste_repository::fhir::CachePolicy::NoCache,
                        )
                        .await?;

                    match &search_request {
                        SearchRequest::Type(_) => {
                            // `_include` reads the references on this page of
                            // matches. It adds entries; it never changes which
                            // resources matched, so `total` is untouched.
                            let directives = search_includes::include_directives(
                                search_request_parameters(&search_request),
                            );

                            let included = search_includes::resolve_includes(
                                state.repo.as_ref(),
                                &ctx.tenant,
                                &ctx.project,
                                &resources,
                                directives,
                            )
                            .await?;

                            let entries = search_entries(&fhir_api_root, resources, included);

                            Ok(FHIRResponse::Search(SearchResponse::Type(
                                FHIRSearchTypeResponse {
                                    bundle: to_bundle(
                                        BundleType::searchset(),
                                        search_results.total,
                                        entries,
                                    ),
                                },
                            )))
                        }
                        SearchRequest::System(_) => Ok(FHIRResponse::Search(
                            SearchResponse::System(FHIRSearchSystemResponse {
                                bundle: to_bundle(
                                    BundleType::searchset(),
                                    search_results.total,
                                    resources
                                        .into_iter()
                                        .map(|r| to_bundle_entry(&fhir_api_root, r, None))
                                        .collect(),
                                ),
                            }),
                        )),
                    }
                }
                FHIRRequest::Transaction(transaction_request) => {
                    // Run sort before creating transaction.
                    // So that transaction is only used for the direct submission of the sorted entries.
                    // We want to limit time within a transaction as much as possible.
                    let sorted_transaction = build_sorted_transaction_graph(
                        transaction_request.resource.entry.unwrap_or_default(),
                    )
                    .await?;

                    let transaction_repo = Arc::new(state.repo.transaction(true).await?);

                    let bundle_response = {
                        let transaction_client = FHIRServerClient::new(
                            ServerClientConfig::new(
                                transaction_repo.clone(),
                                state.search.clone(),
                                state.terminology.clone(),
                                state.config.clone(),
                                state.deno_pool.clone(),
                            )
                            .with_audit_repo(state.audit_repo.clone()),
                        );

                        let transaction_context =
                            Arc::new(ctx.swap_client(Arc::new(transaction_client)));

                        process_transaction_bundle(
                            transaction_context.client.as_ref(),
                            transaction_context.clone(),
                            sorted_transaction,
                        )
                        .await
                    };

                    let repo = Arc::try_unwrap(transaction_repo).map_err(|_e| {
                        OperationOutcomeError::fatal(
                            IssueType::exception(),
                            "Failed to unwrap transaction client".to_string(),
                        )
                    })?;

                    match bundle_response {
                        Ok(transaction_bundle) => {
                            repo.commit().await?;
                            Ok(FHIRResponse::Transaction(FHIRTransactionResponse {
                                resource: transaction_bundle,
                            }))
                        }
                        Err(operation_error) => {
                            tracing::info!("Rolling back transaction due to error");
                            repo.rollback().await?;
                            Err(operation_error)
                        }
                    }
                }
                FHIRRequest::Batch(batch_request) => {
                    let batch_client = FHIRServerClient::new(
                        ServerClientConfig::new(
                            state.repo.clone(),
                            state.search.clone(),
                            state.terminology.clone(),
                            state.config.clone(),
                            state.deno_pool.clone(),
                        )
                        .with_audit_repo(state.audit_repo.clone()),
                    );

                    let batch_context = Arc::new(ctx.swap_client(Arc::new(batch_client)));

                    Ok(FHIRResponse::Batch(FHIRBatchResponse {
                        resource: process_batch_bundle(
                            batch_context.client.as_ref(),
                            batch_context.clone(),
                            batch_request.resource.entry.unwrap_or_default(),
                        )
                        .await?,
                    }))
                }
                FHIRRequest::Patch(patch_request) => {
                    let patched_resource = apply_patch(
                        state.repo.as_ref(),
                        &ctx.tenant,
                        &ctx.project,
                        &patch_request,
                    )
                    .await?;

                    Ok(FHIRResponse::Patch(FHIRPatchResponse {
                        resource: FHIRRepository::update(
                            state.repo.as_ref(),
                            &ctx.tenant,
                            &ctx.project,
                            &ctx.user.claims,
                            &ctx.fhir_version,
                            patched_resource,
                            &patch_request.id,
                        )
                        .await?,
                    }))
                }
                FHIRRequest::Capabilities | FHIRRequest::Invocation(_) => {
                    Err(OperationOutcomeError::error(
                        IssueType::not_supported(),
                        "Unsupported FHIR operation".to_string(),
                    ))
                }
                FHIRRequest::Compartment(compartment_request) => {
                    process_compartment_request(
                        ctx.client.as_ref(),
                        ctx.clone(),
                        &compartment_request,
                    )
                    .await
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use haste_fhir_model::r4::generated::resources::Patient;

    /// `fullUrl` has to address the resource under the FHIR root. The root's
    /// last segment is `fhir`, which `Url::join` would treat as the document to
    /// resolve against and drop.
    #[test]
    fn bundle_entry_full_url_addresses_the_resource() {
        let root = Url::parse("https://api.haste.health/w/acme/default/api/v1/fhir").unwrap();

        let entry = to_bundle_entry(
            &root,
            Resource::Patient(Patient {
                id: Some("123".to_string()),
                ..Default::default()
            }),
            None,
        );

        assert_eq!(
            entry.fullUrl.and_then(|url| url.value).as_deref(),
            Some("https://api.haste.health/w/acme/default/api/v1/fhir/Patient/123")
        );
    }
}
