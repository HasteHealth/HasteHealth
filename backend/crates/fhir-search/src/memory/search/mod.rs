//! A search backend over resources held in memory: the ones the server ships
//! rather than stores, which no index holds. Parameters get the semantics the
//! Postgres and Elasticsearch backends give them; what only a query can do
//! (follow a chain, `_sort`) is refused as not supported.
//!
//! A search is a pipeline of pure functions over a snapshot of the resources:
//! the parameters become criteria and a page, each resource is tested against
//! the criteria, and the page is cut from the matches. Only evaluating a
//! parameter's `FHIRPath` expression is async, so the pass over the resources
//! goes through the small async combinators at the end of this file. They
//! take an `async fn` and its context rather than a closure: a closure held
//! across an await can't be proven `Send` (rust-lang/rust#102211).

mod predicate;

use std::sync::{Arc, PoisonError, RwLock};

use crate::{
    IndexOutcome, IndexResource, MAX_COUNT, ResolvedParameter, SearchEngine, SearchEntry,
    SearchOptions, SearchReturn,
    indexing_conversion::to_insertable_index,
    memory::R4_SEARCH_PARAMETERS_INDEX,
    query::{QueryBuildError, max_count, parse_u64, type_filter, value::Values},
};
use haste_fhir_client::{
    request::SearchRequest,
    url::{Parameter, ParsedParameter, ParsedParameters},
};
use haste_fhir_model::r4::generated::resources::{Resource, ResourceType};
use haste_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use haste_fhirpath::FPEngine;
use haste_jwt::{ProjectId, ResourceId, TenantId, VersionId};
use haste_reflect::MetaValue;
use haste_repository::types::{FHIRMethod, SupportedFHIRVersions};
use predicate::Predicate;

#[derive(OperationOutcomeError, Debug)]
enum MemorySearchError {
    #[fatal(
        code = "exception",
        diagnostic = "Failed to evaluate fhirpath expression."
    )]
    FHIRPath(#[from] haste_fhirpath::FHIRPathError),
}

/// A resource and what a search returns for it.
#[derive(Clone)]
struct Stored {
    id: ResourceId,
    version_id: VersionId,
    resource_type: ResourceType,
    resource: Resource,
}

/// One resource parameter: the expression that finds a resource's values,
/// and what they must satisfy. The definition is borrowed from the R4
/// parameter index, which lives for the program; the values from the request.
struct Criterion<'a> {
    definition: &'static ResolvedParameter,
    expression: &'static str,
    predicate: Predicate<'a>,
}

/// What a resource must satisfy: every criterion, and one of `any_of`'s when
/// the search restricts it.
struct Criteria<'a> {
    all: Vec<Criterion<'a>>,
    any_of: Option<Vec<Criterion<'a>>>,
}

/// The page the result parameters select. The total is always known in
/// memory, so `_total` only has its value checked.
struct Page {
    count: u64,
    offset: u64,
}

/// Resources held at the system level: the same in every tenant and project.
pub struct MemorySearch {
    resources: RwLock<Arc<Vec<Stored>>>,
}

impl MemorySearch {
    /// A backend over `resources`, searched by the R4 search parameters. Each
    /// is stored as version `1`; one without an id is left out, since no
    /// search could return it.
    #[must_use]
    pub fn new(resources: impl IntoIterator<Item = Resource>) -> Self {
        let stored = resources
            .into_iter()
            .filter_map(|resource| stored(resource, VersionId::new("1".to_string())))
            .collect();

        MemorySearch {
            resources: RwLock::new(Arc::new(stored)),
        }
    }
}

impl SearchEngine for MemorySearch {
    async fn search(
        &self,
        _fhir_version: &SupportedFHIRVersions,
        _tenant: &TenantId,
        project: &ProjectId,
        search_request: &SearchRequest,
        options: Option<SearchOptions>,
    ) -> Result<SearchReturn, OperationOutcomeError> {
        // A snapshot, so the lock isn't held while expressions evaluate.
        let resources = self
            .resources
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        search(&resources, project, search_request, options).await
    }

    /// Adds, replaces or removes each resource by type and id. The resources
    /// are system-level, so the tenant and project aren't recorded.
    fn index(
        &self,
        _fhir_version: SupportedFHIRVersions,
        resources: Vec<IndexResource>,
    ) -> impl Future<Output = Result<IndexOutcome, OperationOutcomeError>> + Send {
        let succeeded = resources.len();
        {
            let mut current = self
                .resources
                .write()
                .unwrap_or_else(PoisonError::into_inner);
            let next = indexed(&current, resources);
            *current = Arc::new(next);
        }

        std::future::ready(Ok(IndexOutcome {
            succeeded,
            failed: Vec::new(),
        }))
    }

    /// Nothing to migrate: there is no index.
    fn migrate(
        &self,
        _fhir_version: &SupportedFHIRVersions,
    ) -> impl Future<Output = Result<(), OperationOutcomeError>> + Send {
        std::future::ready(Ok(()))
    }
}

/// `resource` stored at `version_id`; `None` for one without an id, which no
/// search could return.
fn stored(resource: Resource, version_id: VersionId) -> Option<Stored> {
    let id = resource
        .get_field("id")?
        .as_any()
        .downcast_ref::<String>()?
        .clone();
    Some(Stored {
        id: ResourceId::new(id),
        version_id,
        resource_type: resource.resource_type(),
        resource,
    })
}

/// `existing` with each of `batch` added, replaced or removed by type and id,
/// in order.
fn indexed(existing: &[Stored], batch: Vec<IndexResource>) -> Vec<Stored> {
    batch
        .into_iter()
        .fold(existing.to_vec(), |mut stored, resource| {
            stored.retain(|current| {
                current.resource_type != resource.resource_type
                    || current.id.as_ref() != resource.id.as_ref()
            });
            if !matches!(resource.fhir_method, FHIRMethod::Delete) {
                stored.push(Stored {
                    id: resource.id,
                    version_id: resource.version_id,
                    resource_type: resource.resource_type,
                    resource: resource.resource,
                });
            }
            stored
        })
}

/// The page of `resources` the request selects, with the total. The entries
/// are stamped with `project`: the resources are the same in every project.
async fn search(
    resources: &[Stored],
    project: &ProjectId,
    request: &SearchRequest,
    options: Option<SearchOptions>,
) -> Result<SearchReturn, OperationOutcomeError> {
    let (resource_type, parameters) = request_parts(request);
    let count = max_count(options.as_ref())?;
    let any_of = options.as_ref().and_then(|options| options.any_of.as_ref());

    let criteria = Criteria {
        all: parameters
            .parameters()
            .iter()
            .filter_map(resource_parameter)
            .map(|parameter| criterion(resource_type, parameter))
            .collect::<Result<_, _>>()?,
        // `ANDed` with the client's parameters; empty matches nothing.
        any_of: any_of
            .map(|any_of| {
                any_of
                    .0
                    .iter()
                    .map(|parameter| criterion(resource_type, parameter))
                    .collect::<Result<_, _>>()
            })
            .transpose()?,
    };
    let page = parameters
        .parameters()
        .iter()
        .filter_map(result_parameter)
        .try_fold(Page { count, offset: 0 }, paged)?;

    let types = type_filter(request)?;
    let candidates = resources
        .iter()
        .filter(|stored| resource_type.is_none_or(|rt| *rt == stored.resource_type))
        .filter(|stored| {
            types
                .as_ref()
                .is_none_or(|types| types.contains(&stored.resource_type))
        })
        .collect::<Vec<_>>();
    let hits = retained(candidates, &criteria, matched).await?;

    Ok(SearchReturn {
        total: Some(i64::try_from(hits.len()).unwrap_or(i64::MAX)),
        entries: hits
            .into_iter()
            .skip(to_usize(page.offset))
            .take(to_usize(page.count))
            .map(|stored| entry(stored, project))
            .collect(),
    })
}

fn request_parts(request: &SearchRequest) -> (Option<&ResourceType>, &ParsedParameters) {
    match request {
        SearchRequest::Type(request) => (Some(&request.resource_type), &request.parameters),
        SearchRequest::System(request) => (None, &request.parameters),
    }
}

fn resource_parameter(parameter: &ParsedParameter) -> Option<&Parameter> {
    match parameter {
        ParsedParameter::Resource(parameter) => Some(parameter),
        ParsedParameter::Result(_) => None,
    }
}

fn result_parameter(parameter: &ParsedParameter) -> Option<&Parameter> {
    match parameter {
        ParsedParameter::Result(parameter) => Some(parameter),
        ParsedParameter::Resource(_) => None,
    }
}

/// The criterion for one resource parameter on `resource_type`.
fn criterion<'a>(
    resource_type: Option<&ResourceType>,
    parameter: &'a Parameter,
) -> Result<Criterion<'a>, OperationOutcomeError> {
    // Following a reference needs the referenced resource, which only a
    // query has.
    if parameter.is_chained() {
        return Err(QueryBuildError::UnsupportedParameter(parameter.key()).into());
    }

    let definition = R4_SEARCH_PARAMETERS_INDEX
        .get(resource_type, &parameter.name)
        .ok_or_else(|| QueryBuildError::MissingParameter(parameter.name.clone()))?;
    let search_parameter = &definition.search_parameter;
    let expression = search_parameter
        .expression
        .as_ref()
        .and_then(|expression| expression.value.as_deref())
        .ok_or_else(|| QueryBuildError::UnsupportedParameter(parameter.name.clone()))?;

    Ok(Criterion {
        definition,
        expression,
        predicate: Predicate::new(Values::parse(parameter, &search_parameter.type_)?)?,
    })
}

/// `page` with one result parameter applied.
fn paged(page: Page, parameter: &Parameter) -> Result<Page, OperationOutcomeError> {
    Ok(match parameter.name.as_str() {
        "_count" => Page {
            count: parse_u64(parameter)?.min(MAX_COUNT),
            ..page
        },
        "_offset" => Page {
            offset: parse_u64(parameter)?,
            ..page
        },
        "_total" => match parameter.value.as_slice() {
            [value] if matches!(value.as_str(), "none" | "estimate" | "accurate") => page,
            _ => return Err(QueryBuildError::InvalidParameterValue(parameter.name.clone()).into()),
        },
        // Handled in middleware, as for the other backends.
        "_summary" | "_elements" | "_include" | "_type" => page,
        // `_sort` and the rest need what only a query has.
        _ => return Err(QueryBuildError::UnsupportedParameter(parameter.name.clone()).into()),
    })
}

/// Whether `stored` satisfies the criteria.
async fn matched(stored: &Stored, criteria: &Criteria<'_>) -> Result<bool, OperationOutcomeError> {
    let resource = &stored.resource;
    Ok(every(&criteria.all, resource, matches).await?
        && if let Some(any_of) = &criteria.any_of {
            some(any_of, resource, matches).await?
        } else {
            true
        })
}

/// Whether `resource`'s values for the criterion's parameter satisfy the
/// searched values.
async fn matches(
    criterion: &Criterion<'_>,
    resource: &Resource,
) -> Result<bool, OperationOutcomeError> {
    let found = FPEngine::new()
        .evaluate(criterion.expression, vec![resource])
        .await
        .map_err(MemorySearchError::from)?;
    let index = to_insertable_index(criterion.definition, &found.iter().collect::<Vec<_>>())?;
    Ok(criterion.predicate.holds(&index))
}

/// What a search returns for `stored`, in `project`.
fn entry(stored: &Stored, project: &ProjectId) -> SearchEntry {
    SearchEntry {
        id: stored.id.clone(),
        resource_type: stored.resource_type.clone(),
        version_id: stored.version_id.clone(),
        project: project.clone(),
    }
}

/// A page bound as a position in the matches; saturates on a 32-bit target.
fn to_usize(bound: u64) -> usize {
    usize::try_from(bound).unwrap_or(usize::MAX)
}

/// Whether `test` holds for every item, stopping at the first it fails.
async fn every<T, C: Copy, E>(
    items: impl IntoIterator<Item = T>,
    context: C,
    test: impl AsyncFn(T, C) -> Result<bool, E>,
) -> Result<bool, E> {
    for item in items {
        if !test(item, context).await? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Whether `test` holds for some item, stopping at the first it passes.
async fn some<T, C: Copy, E>(
    items: impl IntoIterator<Item = T>,
    context: C,
    test: impl AsyncFn(T, C) -> Result<bool, E>,
) -> Result<bool, E> {
    for item in items {
        if test(item, context).await? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// The items `keep` holds for, in order.
async fn retained<T: Copy, C: Copy, E>(
    items: impl IntoIterator<Item = T>,
    context: C,
    keep: impl AsyncFn(T, C) -> Result<bool, E>,
) -> Result<Vec<T>, E> {
    let mut accepted = Vec::new();
    for item in items {
        if keep(item, context).await? {
            accepted.push(item);
        }
    }
    Ok(accepted)
}
