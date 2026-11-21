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
        ClientState, ServerCTX,
        middleware::{Backend, Middleware, Next, Output, storage::to_bundle_entry},
    },
    route_path::api_fhir_root_url,
};
use haste_fhir_client::request::{
    DeleteRequest, FHIRReadResponse, FHIRRequest, FHIRResponse, FHIRSearchTypeRequest,
    SearchRequest, SearchResponse, UpdateRequest,
};
use haste_fhir_client::url::{Parameter, ParsedParameter, ParsedParameters};
use haste_fhir_model::r4::generated::{
    resources::{BundleEntry, BundleEntrySearch, OperationDefinition, Resource, ResourceType},
    terminology::{IssueType, SearchEntryMode},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_ops::overlaps;
use haste_fhir_search::{
    DEFAULT_COUNT, MAX_COUNT, SearchEngine, SearchReturn, memory::search::MemorySearch,
};
use std::sync::Arc;

/// The OperationDefinitions of the server's built-in operations.
pub type Builtins = Arc<[&'static OperationDefinition]>;

pub struct OperationDefinitions {
    builtins: Builtins,
    search: MemorySearch,
}

impl OperationDefinitions {
    pub fn new(builtins: Builtins) -> Self {
        let search = MemorySearch::new(
            builtins
                .iter()
                .map(|builtin| Resource::OperationDefinition((*builtin).clone())),
        );
        OperationDefinitions { builtins, search }
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

impl<B: Backend> Middleware<B> for OperationDefinitions {
    fn call<'a>(
        &'a self,
        state: &'a ClientState<B>,
        ctx: Arc<ServerCTX<B>>,
        request: FHIRRequest,
        next: Next<'a, B>,
    ) -> Output<'a> {
        Box::pin(async move {
            match &request {
                FHIRRequest::Create(create) => check_definition(&self.builtins, &create.resource)?,
                FHIRRequest::Update(UpdateRequest::Instance(update)) => {
                    check_id(&self.builtins, &update.id)?;
                    check_definition(&self.builtins, &update.resource)?;
                }
                FHIRRequest::Update(UpdateRequest::Conditional(update)) => {
                    check_definition(&self.builtins, &update.resource)?;
                }
                FHIRRequest::Delete(DeleteRequest::Instance(delete)) => {
                    check_id(&self.builtins, &delete.id)?;
                }
                FHIRRequest::Read(read) => {
                    if let Some(builtin) = by_id(&self.builtins, &read.id) {
                        return Ok(FHIRResponse::Read(FHIRReadResponse {
                            resource: Some(Resource::OperationDefinition(builtin.clone())),
                        }));
                    }
                }
                FHIRRequest::Search(search @ SearchRequest::Type(type_request)) => {
                    let found = match self
                        .search
                        .search(&ctx.fhir_version, &ctx.tenant, &ctx.project, search, None)
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
                        .filter_map(|entry| by_id(&self.builtins, entry.id.as_ref()))
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

                    // The project's share of the page.
                    let project_search =
                        FHIRRequest::Search(SearchRequest::Type(FHIRSearchTypeRequest {
                            resource_type: ResourceType::OperationDefinition,
                            parameters: project_parameters,
                        }));
                    let mut response = next.run(state, ctx, project_search).await?;

                    if let FHIRResponse::Search(SearchResponse::Type(search_response)) =
                        &mut response
                    {
                        let bundle = &mut search_response.bundle;
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

                    return Ok(response);
                }
                _ => {}
            }

            next.run(state, ctx, request).await
        })
    }
}
