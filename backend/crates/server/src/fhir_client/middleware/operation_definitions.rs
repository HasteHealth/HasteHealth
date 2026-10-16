//! The OperationDefinitions of the server's built-in operations, which each
//! generated operation module carries. They ship with the server rather than
//! live in a project, so this middleware serves them beside the project's own:
//! a read of a built-in's id returns it, and a type search lists the built-ins
//! it matches (found by an in-memory search backend) ahead of the project's.
//!
//! Built-in implementations run first, so a project OperationDefinition that
//! overlaps one (the same code, at a level and on a resource type they share)
//! would never run, and one with a built-in's id or url would shadow it on a
//! read or a canonical lookup. Writes that would do either are refused; a
//! built-in's code is free for other levels and resource types.
use crate::{
    fhir_client::{
        ServerCTX,
        middleware::{
            ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput,
            ServerMiddlewareState, storage::to_bundle_entry,
        },
    },
    route_path::api_fhir_root_url,
};
use haste_fhir_client::{
    FHIRClient,
    middleware::MiddlewareChain,
    request::{
        DeleteRequest, FHIRReadResponse, FHIRRequest, FHIRResponse, FHIRSearchTypeRequest,
        SearchRequest, SearchResponse, UpdateRequest,
    },
    url::{Parameter, ParsedParameter, ParsedParameters},
};
use haste_fhir_model::r4::generated::{
    resources::{BundleEntry, BundleEntrySearch, OperationDefinition, Resource, ResourceType},
    terminology::{IssueType, SearchEntryMode},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_ops::overlaps;
use haste_fhir_search::{
    DEFAULT_COUNT, MAX_COUNT, SearchEngine, SearchReturn, memory::search::MemorySearch,
};
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::Repository;
use std::sync::Arc;

/// The OperationDefinitions of the server's built-in operations.
pub type Builtins = Arc<[&'static OperationDefinition]>;

pub struct Middleware {
    builtins: Builtins,
    search: Arc<MemorySearch>,
}

impl Middleware {
    pub fn new(builtins: Builtins) -> Self {
        let search = MemorySearch::new(
            builtins
                .iter()
                .map(|builtin| Resource::OperationDefinition((*builtin).clone())),
        );
        Middleware {
            builtins,
            search: Arc::new(search),
        }
    }
}

fn by_id(builtins: &Builtins, id: &str) -> Option<&'static OperationDefinition> {
    builtins
        .iter()
        .copied()
        .find(|builtin| builtin.id.as_deref() == Some(id))
}

fn canonical(definition: &OperationDefinition) -> Option<&str> {
    definition.url.as_ref().and_then(|url| url.value.as_deref())
}

fn conflict(message: String) -> OperationOutcomeError {
    OperationOutcomeError::error(IssueType::conflict(), message)
}

/// Refuses a write to a built-in's id.
fn check_id(builtins: &Builtins, id: &str) -> Result<(), OperationOutcomeError> {
    match by_id(builtins, id) {
        Some(_) => Err(conflict(format!(
            "OperationDefinition '{id}' is built in and cannot be changed or deleted."
        ))),
        None => Ok(()),
    }
}

/// Refuses an OperationDefinition that overlaps a built-in or shares its url.
fn check_definition(builtins: &Builtins, resource: &Resource) -> Result<(), OperationOutcomeError> {
    let Resource::OperationDefinition(definition) = resource else {
        return Ok(());
    };

    if let Some(builtin) = builtins
        .iter()
        .find(|builtin| overlaps(builtin, definition))
    {
        return Err(conflict(format!(
            "The built-in operation '{}' already handles code '{}' at a level and on a \
             resource type this definition allows. Use a different code, or narrow the \
             definition's levels or resource types.",
            builtin.id.as_deref().unwrap_or_default(),
            definition.code.value.as_deref().unwrap_or_default(),
        )));
    }

    if let Some(url) = canonical(definition)
        && builtins
            .iter()
            .any(|builtin| canonical(builtin) == Some(url))
    {
        return Err(conflict(format!(
            "The url '{url}' belongs to a built-in operation. Use a different url."
        )));
    }

    Ok(())
}

/// Whether `error` only says the memory backend can't run the search (a
/// chain, `_sort`, a composite), which the project search then runs or
/// reports.
fn not_supported(error: &OperationOutcomeError) -> bool {
    error
        .outcome()
        .issue
        .iter()
        .all(|issue| issue.code == IssueType::not_supported())
}

/// The parameters for the project's share of the page: the client's, with
/// `_offset` and `_count` reduced by the built-in matches before and on it.
fn rest(parameters: &ParsedParameters, builtins: u64, shown: u64) -> ParsedParameters {
    let number = |name: &str| match parameters.get(name) {
        Some(ParsedParameter::Result(parameter)) => parameter
            .value
            .first()
            .and_then(|value| value.parse::<u64>().ok()),
        _ => None,
    };
    let offset = number("_offset").unwrap_or(0);
    let count = number("_count").map_or(DEFAULT_COUNT, |count| count.min(MAX_COUNT));

    let result = |name: &str, value: u64| {
        ParsedParameter::Result(Parameter::from((name.to_string(), vec![value.to_string()])))
    };
    ParsedParameters::new(
        parameters
            .parameters()
            .iter()
            .filter(|parameter| !matches!(parameter.name(), "_offset" | "_count"))
            .cloned()
            .chain([
                result("_offset", offset.saturating_sub(builtins)),
                result("_count", count.saturating_sub(shown)),
            ])
            .collect(),
    )
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
    > for Middleware
{
    fn call(
        &self,
        state: ServerMiddlewareState<Repo, Search, Terminology>,
        mut context: ServerMiddlewareContext<Client>,
        next: Option<
            Arc<ServerMiddlewareNext<Client, ServerMiddlewareState<Repo, Search, Terminology>>>,
        >,
    ) -> ServerMiddlewareOutput<Client> {
        let builtins = self.builtins.clone();
        let search = self.search.clone();

        Box::pin(async move {
            let Some(next) = next else {
                return Err(OperationOutcomeError::fatal(
                    IssueType::exception(),
                    "No next middleware found".to_string(),
                ));
            };

            match &context.request {
                FHIRRequest::Create(create) => {
                    check_definition(&builtins, &create.resource)?;
                    next(state, context).await
                }
                FHIRRequest::Update(UpdateRequest::Instance(update)) => {
                    check_id(&builtins, &update.id)?;
                    check_definition(&builtins, &update.resource)?;
                    next(state, context).await
                }
                FHIRRequest::Update(UpdateRequest::Conditional(update)) => {
                    check_definition(&builtins, &update.resource)?;
                    next(state, context).await
                }
                FHIRRequest::Delete(DeleteRequest::Instance(delete)) => {
                    check_id(&builtins, &delete.id)?;
                    next(state, context).await
                }
                FHIRRequest::Read(read) => match by_id(&builtins, &read.id) {
                    Some(builtin) => {
                        context.response = Some(FHIRResponse::Read(FHIRReadResponse {
                            resource: Some(Resource::OperationDefinition(builtin.clone())),
                        }));
                        Ok(context)
                    }
                    None => next(state, context).await,
                },
                FHIRRequest::Search(request @ SearchRequest::Type(type_request)) => {
                    let ctx = &context.ctx;
                    let found = match search
                        .search(&ctx.fhir_version, &ctx.tenant, &ctx.project, request, None)
                        .await
                    {
                        Ok(found) => found,
                        Err(error) if not_supported(&error) => SearchReturn {
                            total: Some(0),
                            entries: Vec::new(),
                        },
                        Err(error) => return Err(error),
                    };
                    let builtin_total = found
                        .total
                        .and_then(|total| u64::try_from(total).ok())
                        .unwrap_or(0);
                    let project_parameters = rest(
                        &type_request.parameters,
                        builtin_total,
                        found.entries.len() as u64,
                    );

                    let fhir_api_root =
                        api_fhir_root_url(&state.config.api_uri, &ctx.tenant, &ctx.project)?;
                    let builtin_entries = found
                        .entries
                        .iter()
                        .filter_map(|entry| by_id(&builtins, entry.id.as_ref()))
                        .map(|builtin| BundleEntry {
                            search: Some(BundleEntrySearch {
                                mode: Some(SearchEntryMode::match_()),
                                ..Default::default()
                            }),
                            ..to_bundle_entry(
                                &fhir_api_root,
                                Resource::OperationDefinition(builtin.clone()),
                                None,
                            )
                        })
                        .collect::<Vec<_>>();

                    // Search the project for the rest of the page, then put
                    // the caller's request back.
                    let request = std::mem::replace(
                        &mut context.request,
                        FHIRRequest::Search(SearchRequest::Type(FHIRSearchTypeRequest {
                            resource_type: ResourceType::OperationDefinition,
                            parameters: project_parameters,
                        })),
                    );
                    let mut context = next(state, context).await?;
                    context.request = request;

                    if let Some(FHIRResponse::Search(SearchResponse::Type(response))) =
                        context.response.as_mut()
                    {
                        let bundle = &mut response.bundle;
                        bundle.entry = Some(
                            builtin_entries
                                .into_iter()
                                .chain(bundle.entry.take().unwrap_or_default())
                                .collect(),
                        );
                        if let Some(total) = bundle.total.as_mut() {
                            total.value = total.value.map(|total| total + builtin_total);
                        }
                    }

                    Ok(context)
                }
                _ => next(state, context).await,
            }
        })
    }
}
