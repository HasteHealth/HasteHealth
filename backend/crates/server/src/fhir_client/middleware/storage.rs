use axum::http::Method;
use oxidized_fhir_client::{
    FHIRClient,
    request::{
        FHIRCreateResponse, FHIRHistoryInstanceResponse, FHIRReadResponse, FHIRRequest,
        FHIRResponse, FHIRSearchSystemResponse, FHIRSearchTypeRequest, FHIRSearchTypeResponse,
        FHIRUpdateResponse, FHIRVersionReadResponse,
    },
    url::ParsedParameter,
};
use oxidized_fhir_model::r4::types::{Bundle, FHIRCode};

use crate::{
    fhir_client::{
        ClientState, FHIRServerClient, StorageError,
        middleware::{
            ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput,
            ServerMiddlewareState,
        },
    },
    fhir_http::{self, HTTPRequest},
};
use oxidized_fhir_operation_error::{OperationOutcomeCodes, OperationOutcomeError};
use oxidized_fhir_search::{SearchEngine, SearchRequest};
use oxidized_reflect::MetaValue;
use oxidized_repository::{
    Repository,
    fhir::{FHIRRepository, HistoryRequest},
    types::{ResourceId, SupportedFHIRVersions, VersionIdRef},
};
use std::{borrow::Cow, str::FromStr, sync::Arc};

pub fn storage<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
>(
    state: ServerMiddlewareState<Repo, Search>,
    mut context: ServerMiddlewareContext,
    next: Option<Arc<ServerMiddlewareNext<Repo, Search>>>,
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
                            .downcast_ref::<Option<String>>()
                            .unwrap()
                            .clone();

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
                                    OperationOutcomeCodes::NotFound,
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
                                OperationOutcomeCodes::Conflict,
                                "Resource type mismatch".to_string(),
                            ));
                        }

                        let resource_id_body = update_request
                            .resource
                            .get_field("id")
                            .ok_or_else(|| {
                                OperationOutcomeError::error(
                                    OperationOutcomeCodes::Invalid,
                                    "Missing resource ID".to_string(),
                                )
                            })?
                            .as_any()
                            .downcast_ref::<Option<String>>()
                            .unwrap();

                        // If body has resource Id verify it's the same as one in search result.
                        if resource_id_body.is_some()
                            && resource_id_body.as_ref().map(|s| s.as_str())
                                != Some(search_result.id.as_ref())
                        {
                            return Err(OperationOutcomeError::error(
                                OperationOutcomeCodes::Conflict,
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
                        OperationOutcomeCodes::Conflict,
                        "Multiple resources found for conditional update.".to_string(),
                    )),
                }
            }
            FHIRRequest::Batch(batch_request) => {
                let mut bundle_entries = Some(Vec::new());
                // Memswap so I can avoid cloning.
                std::mem::swap(&mut batch_request.resource.entry, &mut bundle_entries);

                let batch_client = FHIRServerClient::new(state.repo.clone(), state.search.clone());
                let mut response = Bundle {
                    type_: Box::new(FHIRCode {
                        value: Some("batch-response".to_string()),
                        ..Default::default()
                    }),
                    entry: bundle_entries
                        .unwrap_or(vec![])
                        .into_iter()
                        .filter_map(|e| {
                            if let Some(request) = e.request.as_ref() {
                                let url = request
                                    .url
                                    .value
                                    .as_ref()
                                    .map(|s| s.as_str())
                                    .unwrap_or_default();

                                let (path, query) = url.split_once("?").unwrap_or((url, ""));

                                let Ok(method) = Method::from_str(
                                    request
                                        .method
                                        .value
                                        .as_ref()
                                        .map(|s| s.as_str())
                                        .unwrap_or_default(),
                                ) else {
                                    return None;
                                };

                                let http_request = HTTPRequest::new(
                                    method,
                                    path.to_string(),
                                    fhir_http::HTTPBody::Resource(*(e.resource.clone().unwrap())),
                                    query.to_string(),
                                );

                                let Ok(fhir_request) = fhir_http::http_request_to_fhir_request(
                                    SupportedFHIRVersions::R4,
                                    http_request,
                                ) else {
                                    return None;
                                };

                                batch_client.request(context.ctx.clone(), fhir_request);

                                // BundleEntry {
                                //     response: fhir_request,
                                // };
                            }

                            todo!();
                        })
                        .collect(),
                    ..Default::default()
                };

                todo!();
            }
            _ => Ok(None),
        }?;

        let mut next_context = if let Some(next_) = next {
            next_(
                Arc::new(ClientState {
                    repo: Arc::new(state.repo.transaction().await.unwrap()),
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
