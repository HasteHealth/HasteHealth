use base64::{Engine as _, engine::general_purpose};
use futures::future::{BoxFuture, try_join, try_join_all};
use haste_fhir_client::{FHIRClient, url::ParsedParameters};
use haste_fhir_generated_ops::generated::ViewDefinitionRun;
use haste_fhir_model::r4::{
    datetime::Instant,
    generated::{
        resources::{
            Binary, BundleEntry, Resource, ResourceType, ViewDefinition, ViewDefinitionSelect,
        },
        terminology::{BoundCode, HttpVerb, IssueType, OutputFormatCodes},
        types::{FHIRBase64Binary, FHIRBoolean, FHIRCode, FHIRInstant, FHIRString, Reference},
    },
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhirpath::{Config, Context, FPEngine};
use haste_reflect::MetaValue;
use itertools::Itertools as _;
use ordermap::OrderMap;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::conversions::primitives::PrimitiveValue;

mod compartment;
mod conversions;
mod output;

/// An undefined constant or an unparsable expression is the caller's mistake; any other
/// `FHIRPath` failure stays a server error.
fn fhirpath_issue_type(e: &haste_fhirpath::FHIRPathError) -> BoundCode<IssueType> {
    match e {
        haste_fhirpath::FHIRPathError::UndefinedConstant(_)
        | haste_fhirpath::FHIRPathError::ParseError(_) => IssueType::invalid(),
        _ => IssueType::exception(),
    }
}

fn reference_value(reference: &Reference) -> Result<String, OperationOutcomeError> {
    reference
        .reference
        .as_ref()
        .and_then(|r| r.value.clone())
        .ok_or_else(|| {
            OperationOutcomeError::error(
                IssueType::invalid(),
                "Reference.reference is required".to_string(),
            )
        })
}

fn invalid(message: String) -> OperationOutcomeError {
    OperationOutcomeError::error(IssueType::invalid(), message)
}

/// The id a `<resource_type>/<id>` reference names, if it names one of that type.
fn reference_id<'a>(reference: &'a str, resource_type: &str) -> Option<&'a str> {
    reference
        .strip_prefix(resource_type)
        .and_then(|rest| rest.strip_prefix('/'))
        .and_then(|rest| rest.split('/').next())
        .filter(|id| !id.is_empty())
}

/// Reads the current version of each `(type, id)` in one round trip, leaving out the keys that
/// don't resolve. The server backs it with the repository's batch read, so a run's `patient` and
/// `group` lookups cost one query however many Groups are given.
pub type ReadMany<CTX> = Arc<
    dyn Fn(
            CTX,
            Vec<(ResourceType, String)>,
        ) -> BoxFuture<'static, Result<Vec<Resource>, OperationOutcomeError>>
        + Send
        + Sync,
>;

/// Keys one batch read takes.
const SCOPE_KEYS_PER_READ: usize = 1000;

/// The id the scope `parameter` names, refused when it isn't a reference to `resource_type`.
fn scope_id(
    parameter: &str,
    reference: &Reference,
    resource_type: &ResourceType,
) -> Result<String, OperationOutcomeError> {
    let value = reference_value(reference)?;
    reference_id(&value, resource_type.as_ref())
        .map(str::to_string)
        .ok_or_else(|| {
            invalid(format!(
                "'{parameter}' must reference a {}, not '{value}'.",
                resource_type.as_ref()
            ))
        })
}

/// The patients a run is limited to: `None` without `patient` or `group` (or with resources sent
/// inline), else the ids of the Patients every given parameter allows. Both are limits, so with
/// both a run covers only the `patient` and only if it is in a group. An empty list means no
/// resource qualifies.
///
/// The Patient and the Groups are read in one batch; one the server doesn't have is refused.
/// Members that aren't Patients are left out: only the Patient compartment is supported.
async fn patient_scope<CTX: Clone>(
    context: CTX,
    read_many: &ReadMany<CTX>,
    input: &ViewDefinitionRun::Input,
) -> Result<Option<Vec<String>>, OperationOutcomeError> {
    if input.resource.is_some() {
        return Ok(None);
    }

    let patient = input
        .patient
        .as_ref()
        .map(|reference| scope_id("patient", reference, &ResourceType::Patient))
        .transpose()?;
    let groups = input
        .group
        .iter()
        .flatten()
        .map(|reference| scope_id("group", reference, &ResourceType::Group))
        .collect::<Result<Vec<_>, _>>()?;
    if patient.is_none() && groups.is_empty() {
        return Ok(None);
    }

    let keys = patient
        .iter()
        .map(|id| (ResourceType::Patient, id.clone()))
        .chain(groups.iter().map(|id| (ResourceType::Group, id.clone())))
        .collect::<Vec<_>>();
    let mut found = HashMap::new();
    for chunk in keys.chunks(SCOPE_KEYS_PER_READ) {
        for resource in read_many(context.clone(), chunk.to_vec()).await? {
            if let Some(id) = resource.id().clone() {
                found.insert((resource.resource_type(), id), resource);
            }
        }
    }

    if let Some(id) = &patient
        && !found.contains_key(&(ResourceType::Patient, id.clone()))
    {
        return Err(invalid(format!("Patient with id '{id}' not found.")));
    }

    let mut members = Vec::new();
    for id in &groups {
        let Some(Resource::Group(group)) = found.get(&(ResourceType::Group, id.clone())) else {
            return Err(invalid(format!("Group with id '{id}' not found.")));
        };
        members.extend(group.member.iter().flatten().filter_map(|member| {
            let entity = member.entity.reference.as_ref()?.value.as_ref()?;
            reference_id(entity, ResourceType::Patient.as_ref()).map(str::to_string)
        }));
    }
    members.sort_unstable();
    members.dedup();

    Ok(Some(match patient {
        Some(patient) if groups.is_empty() => vec![patient],
        Some(patient) => members
            .into_iter()
            .filter(|member| *member == patient)
            .collect(),
        None => members,
    }))
}

async fn resolve_view_definition<
    'a,
    CTX: Send + Sync + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + Send + Sync + 'static,
>(
    context: CTX,
    client: &Client,
    input: &'a ViewDefinitionRun::Input,
) -> Result<Cow<'a, ViewDefinition>, OperationOutcomeError> {
    if input.viewResource.is_some() && input.viewReference.is_some() {
        return Err(invalid(
            "Only one of viewResource and viewReference may be provided.".to_string(),
        ));
    }

    if let Some(view_definition) = &input.viewResource {
        Ok(Cow::Borrowed(view_definition))
    } else if let Some(view_definition_reference) = input.viewReference.as_ref() {
        let view_definition_reference = view_definition_reference
            .reference
            .as_ref()
            .ok_or_else(|| {
                OperationOutcomeError::error(
                    IssueType::invalid(),
                    "viewReference.reference is required".to_string(),
                )
            })?
            .value
            .as_ref()
            .ok_or_else(|| {
                OperationOutcomeError::error(
                    IssueType::invalid(),
                    "viewReference.reference.value is required".to_string(),
                )
            })?;

        let reference_pieces = view_definition_reference.split('/').collect::<Vec<_>>();

        let view_definition_id = reference_pieces
            .last()
            .ok_or_else(|| {
                OperationOutcomeError::error(
                    IssueType::invalid(),
                    "Invalid viewReference.reference format".to_string(),
                )
            })?
            .to_string();

        let result = client
            .read(
                context,
                ResourceType::ViewDefinition,
                view_definition_id.clone(),
            )
            .await?
            .ok_or_else(|| {
                OperationOutcomeError::error(
                    IssueType::not_found(),
                    format!("ViewDefinition not found with id '{view_definition_id:?}'"),
                )
            })?;

        if let Resource::ViewDefinition(view_definition) = result {
            Ok(Cow::Owned(view_definition))
        } else {
            Err(OperationOutcomeError::error(
                IssueType::invalid(),
                "Referenced resource is not a ViewDefinition".to_string(),
            ))
        }
    } else {
        Err(OperationOutcomeError::error(
            IssueType::invalid(),
            "Either viewResource or viewReference must be provided".to_string(),
        ))
    }
}

/// Bounds on one run. The operation answers synchronously with one table, so a run that would
/// pass either is refused rather than answered with part of the table.
#[derive(Debug, Clone, Copy)]
pub struct RunLimits {
    /// Most rows a run returns. A larger `_limit` is refused.
    pub max_rows: usize,
    /// Most resources a run reads from storage, earlier versions included.
    pub max_resources: usize,
}

impl Default for RunLimits {
    fn default() -> Self {
        Self {
            max_rows: 10_000,
            max_resources: 50_000,
        }
    }
}

/// Resources fetched per round trip.
const PAGE_SIZE: usize = 1000;

/// Patients one compartment search covers. Larger lists are searched in batches.
const PATIENTS_PER_SEARCH: usize = 100;

/// Deepest one search pages. Elasticsearch refuses `_offset` + `_count` past 10,000 by default
/// (`index.max_result_window`), and deep offsets get slow on Postgres.
const MAX_SEARCH_DEPTH: usize = 10_000;

/// Refuses a run that would pass a limit: 422, as the request is well formed but past this
/// server's limits (the same answer as an `ImplementationGuide/$install` that is too large).
fn too_costly(what: &str) -> OperationOutcomeError {
    let error = OperationOutcomeError::error(
        IssueType::too_costly(),
        format!("{what} Narrow the run with _limit, patient, group or _since."),
    );
    OperationOutcomeError::unprocessable(error.outcome().clone())
}

/// Refuses parameters this server doesn't take, before anything is read.
fn check_parameters(
    input: &ViewDefinitionRun::Input,
    limits: &RunLimits,
) -> Result<(), OperationOutcomeError> {
    if input.source.is_some() {
        return Err(invalid(
            "The server does not support the source parameter.".to_string(),
        ));
    }

    match requested_limit(input) {
        Some(limit) if limit > limits.max_rows => Err(invalid(format!(
            "_limit {limit} is above {}, the most rows one run returns.",
            limits.max_rows
        ))),
        _ => Ok(()),
    }
}

/// What every page of a run is processed with.
struct Run<'v> {
    fp_engine: &'v FPEngine,
    variables: Arc<HashMap<String, &'v dyn MetaValue>>,
    view_definition: &'v ViewDefinition,
    where_clauses: Vec<&'v str>,
    resource_type: ResourceType,
    /// The caller's `_limit`.
    limit: Option<usize>,
    limits: RunLimits,
}

/// Adds the rows of `resources`: those of the view's type that pass its `where`. Returns whether
/// the run has all the rows it asked for. Without `_limit`, more than `max_rows` refuses the run.
async fn add_rows<'v>(
    run: &Run<'v>,
    resources: &[Resource],
    rows: &mut Vec<Row<'v>>,
) -> Result<bool, OperationOutcomeError> {
    for resource in resources_of_type(resources, &run.resource_type) {
        if !passes_where_clauses(
            run.fp_engine,
            run.variables.clone(),
            &run.where_clauses,
            resource,
        )
        .await?
        {
            continue;
        }

        rows.extend(
            process_resource(
                run.fp_engine,
                run.variables.clone(),
                run.view_definition,
                resource,
            )
            .await?,
        );

        match run.limit {
            Some(limit) if rows.len() >= limit => {
                rows.truncate(limit);
                return Ok(true);
            }
            None if rows.len() > run.limits.max_rows => {
                return Err(too_costly(&format!(
                    "The view gives more than {} rows, the most one run returns.",
                    run.limits.max_rows
                )));
            }
            _ => {}
        }
    }

    Ok(false)
}

/// Counts `count` more resources read, refusing the run past `max_resources`.
fn count_read(
    read: &mut usize,
    count: usize,
    limits: &RunLimits,
) -> Result<(), OperationOutcomeError> {
    *read += count;
    if *read > limits.max_resources {
        return Err(too_costly(&format!(
            "The run reads more than {} resources, the most one run reads.",
            limits.max_resources
        )));
    }
    Ok(())
}

/// The view's rows: over the resources sent with the request, else over the server's data,
/// limited to the patients of `scope` (see [`patient_scope`]) and to `_since`.
async fn view_rows<
    'v,
    CTX: Send + Sync + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + Send + Sync + 'static,
>(
    context: CTX,
    client: &Client,
    run: &Run<'v>,
    input: &ViewDefinitionRun::Input,
    scope: Option<Vec<String>>,
) -> Result<Vec<Row<'v>>, OperationOutcomeError> {
    let mut rows = Vec::new();

    if let Some(resources) = &input.resource {
        add_rows(run, resources, &mut rows).await?;
        return Ok(rows);
    }

    let since = input.since.as_ref().and_then(|since| since.value.as_ref());

    match scope {
        None => rows_from_history(context, client, run, since, &mut rows).await?,
        Some(patients) => {
            rows_from_compartments(context, client, run, &patients, since, &mut rows).await?;
        }
    }

    Ok(rows)
}

/// Rows from the current resources of the view's type, newest first. The type's history is read
/// a page at a time; the first entry of an id is its current version, so later ones are skipped.
async fn rows_from_history<
    'v,
    CTX: Send + Sync + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + Send + Sync + 'static,
>(
    context: CTX,
    client: &Client,
    run: &Run<'v>,
    since: Option<&Instant>,
    rows: &mut Vec<Row<'v>>,
) -> Result<(), OperationOutcomeError> {
    let mut seen = HashSet::new();
    let mut read = 0;
    let mut offset = 0;

    loop {
        let mut parameters = vec![
            ("_count".to_string(), vec![PAGE_SIZE.to_string()]),
            ("_offset".to_string(), vec![offset.to_string()]),
        ];
        if let Some(since) = since {
            parameters.push(("_since".to_string(), vec![since.to_string()]));
        }

        let entries = client
            .history_type(
                context.clone(),
                run.resource_type.clone(),
                parameters.into(),
            )
            .await?
            .entry
            .unwrap_or_default();
        let last_page = entries.len() < PAGE_SIZE;

        count_read(&mut read, entries.len(), &run.limits)?;
        let current = current_versions(entries, &mut seen, since);
        if add_rows(run, &current, rows).await? || last_page {
            return Ok(());
        }

        offset += PAGE_SIZE;
    }
}

/// Rows from the compartments of `patients`, in id order. Each compartment parameter is searched
/// a batch of patients and a page at a time. A resource in several compartments counts once.
async fn rows_from_compartments<
    'v,
    CTX: Send + Sync + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + Send + Sync + 'static,
>(
    context: CTX,
    client: &Client,
    run: &Run<'v>,
    patients: &[String],
    since: Option<&Instant>,
    rows: &mut Vec<Row<'v>>,
) -> Result<(), OperationOutcomeError> {
    let mut seen = HashSet::new();
    let mut read = 0;

    for parameter in compartment::patient_search_parameters(&run.resource_type)? {
        for patients in patients.chunks(PATIENTS_PER_SEARCH) {
            let mut offset = 0;

            loop {
                if offset >= MAX_SEARCH_DEPTH {
                    return Err(too_costly(&format!(
                        "More than {MAX_SEARCH_DEPTH} {} resources match one search of these \
                         patients.",
                        run.resource_type.as_ref()
                    )));
                }

                let resources: Vec<Resource> = client
                    .search_type(
                        context.clone(),
                        run.resource_type.clone(),
                        compartment_search(&parameter, patients, since, offset),
                    )
                    .await?
                    .entry
                    .into_iter()
                    .flatten()
                    .filter_map(|entry| entry.resource.map(|resource| *resource))
                    .collect();
                let last_page = resources.len() < PAGE_SIZE;

                count_read(&mut read, resources.len(), &run.limits)?;
                let unseen: Vec<Resource> = resources
                    .into_iter()
                    .filter(|resource| seen.insert(resource.id().clone()))
                    .collect();
                if add_rows(run, &unseen, rows).await? {
                    return Ok(());
                }
                if last_page {
                    break;
                }

                offset += PAGE_SIZE;
            }
        }
    }

    Ok(())
}

/// One page of the resources whose compartment `parameter` names one of `patients`, sorted by id
/// so pages don't overlap, and changed after `since` when given.
fn compartment_search(
    parameter: &str,
    patients: &[String],
    since: Option<&Instant>,
    offset: usize,
) -> ParsedParameters {
    let values = patients
        .iter()
        .map(|id| {
            if parameter == "_id" {
                id.clone()
            } else {
                format!("{}/{id}", ResourceType::Patient.as_ref())
            }
        })
        .collect();

    let mut parameters = vec![
        (parameter.to_string(), values),
        ("_sort".to_string(), vec!["_id".to_string()]),
        ("_count".to_string(), vec![PAGE_SIZE.to_string()]),
        ("_offset".to_string(), vec![offset.to_string()]),
    ];
    if let Some(since) = since {
        parameters.push((
            "_lastUpdated".to_string(),
            vec![format!("gt{}", since.to_string())],
        ));
    }

    parameters.into()
}

/// The client-requested `_limit`, if any: the most output rows to return.
fn requested_limit(input: &ViewDefinitionRun::Input) -> Option<usize> {
    input
        .limit
        .as_ref()
        .and_then(|limit| limit.value)
        .and_then(|limit| usize::try_from(limit).ok())
}

/// The resource type a view runs over (`ViewDefinition.resource`).
fn view_resource_type(
    view_definition: &ViewDefinition,
) -> Result<ResourceType, OperationOutcomeError> {
    let Some(resource_type) = view_definition.resource.as_str() else {
        return Err(OperationOutcomeError::error(
            IssueType::invalid(),
            "ViewDefinition.resource is required".to_string(),
        ));
    };

    ResourceType::try_from(resource_type).map_err(|e| {
        OperationOutcomeError::error(IssueType::invalid(), format!("Invalid resource type: {e}"))
    })
}

/// The current version of each resource in a page of history, newest first as the history is.
/// `seen` carries the ids of earlier pages: a view has one row set per resource, so earlier
/// versions are dropped. A resource whose latest entry is a delete is left out, as is one not
/// changed after `since` (history's `_since` also returns versions made at that instant).
fn current_versions(
    entries: Vec<BundleEntry>,
    seen: &mut HashSet<Option<String>>,
    since: Option<&Instant>,
) -> Vec<Resource> {
    entries
        .into_iter()
        .filter_map(|entry| {
            let resource = entry.resource?;
            if !seen.insert(resource.id().clone()) {
                return None;
            }

            let deleted = entry
                .request
                .is_some_and(|request| request.method == HttpVerb::delete());

            (!deleted && changed_after(&resource, since)).then_some(*resource)
        })
        .collect()
}

/// Whether `resource` changed after `since`. Always without `since`, and for a resource with no
/// `meta.lastUpdated`, as the specification allows.
fn changed_after(resource: &Resource, since: Option<&Instant>) -> bool {
    let Some(Instant::Iso8601(since)) = since else {
        return true;
    };

    resource
        .get_field("meta")
        .and_then(|meta| meta.get_field("lastUpdated"))
        .and_then(|updated| updated.as_any().downcast_ref::<FHIRInstant>())
        .and_then(|updated| updated.value.as_ref())
        .is_none_or(|Instant::Iso8601(updated)| updated > since)
}

fn build_hashmap_fp_variables(viewdefinition: &ViewDefinition) -> HashMap<String, &dyn MetaValue> {
    let mut hashmap = HashMap::new();

    if let Some(constants) = &viewdefinition.constant {
        for constant in constants {
            // Pass the inner primitive: the choice enum's `as_any` is its Box, which number and boolean downcasts reject.
            if let (Some(name), Some(value)) = (
                constant.name.value.as_ref(),
                constant.value.flatten().first().copied(),
            ) {
                hashmap.insert(name.clone(), value);
            }
        }
    }

    hashmap
}

/// Column values by name. Names borrow from the view.
type Row<'a> = OrderMap<&'a str, OutputResults>;

/// Cross product of two row sets, `left`'s columns first.
fn join<'a>(left: &[Row<'a>], right: Vec<Row<'a>>) -> Vec<Row<'a>> {
    let copy = |(&name, value): (&&'a str, &OutputResults)| (name, value.clone());

    match left {
        // The fold's empty start row.
        [row] if row.is_empty() => right,
        // Usual case: move the right rows instead of copying them.
        [row] => right
            .into_iter()
            .map(|r| row.iter().map(copy).chain(r).collect())
            .collect(),
        _ => left
            .iter()
            .cartesian_product(&right)
            .map(|(l, r)| l.iter().chain(r).map(copy).collect())
            .collect(),
    }
}

// Need to distinguish between a scalar value and a collection of values for each column in the output. This enum helps to represent that distinction.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
enum OutputResults {
    Scalar(Option<PrimitiveValue>),
    Collection(Vec<Option<PrimitiveValue>>),
}

/// A resource's rows: the cross product of each select's rows. Rows borrow only from the view
/// (`'v`), not from the resource (`'r`), so a page of resources can go once its rows are made.
async fn process_resource<'r, 'v: 'r>(
    fp_engine: &'r FPEngine,
    variables: Arc<HashMap<String, &'r dyn MetaValue>>,
    view_definition: &'v ViewDefinition,
    input: &'r Resource,
) -> Result<Vec<Row<'v>>, OperationOutcomeError> {
    let fp_config = Arc::new(
        Config::builder()
            .with_variable_resolver(haste_fhirpath::ExternalConstantResolver::Variable(
                variables,
            ))
            .with_resource_id(input.id().clone().unwrap_or_default()),
    );

    let per_select = select_rows(fp_engine, &fp_config, &view_definition.select, input).await?;

    Ok(per_select
        .into_iter()
        .fold(vec![Row::new()], |rows, more| join(&rows, more)))
}

/// Evaluates `path` on `node`; `what` names the expression in errors.
async fn evaluate<'a>(
    fp_engine: &FPEngine,
    fp_config: &Arc<Config<'a>>,
    path: &str,
    node: &'a dyn MetaValue,
    what: &str,
) -> Result<Context<'a>, OperationOutcomeError> {
    fp_engine
        .evaluate_with_config(path, vec![node], fp_config.clone())
        .await
        .map_err(|e| {
            OperationOutcomeError::error(
                fhirpath_issue_type(&e),
                format!("Error evaluating {what}: {e}"),
            )
        })
}

/// Nodes and the evaluations that found them. Computed nodes live in an evaluation's arena, so
/// keep the evaluations while the nodes are in use.
type Nodes<'a> = (Vec<&'a dyn MetaValue>, Vec<Context<'a>>);

fn nodes_of(found: Context<'_>) -> Nodes<'_> {
    (found.values().to_vec(), vec![found])
}

/// Same limit as `FHIRPath` `repeat()`.
const REPEAT_ITEM_LIMIT: usize = 10_000;

/// Items a `repeat` reaches from `input`, depth first in document order. Each item is taken once,
/// so every walk ends.
async fn walk_repeat<'a>(
    fp_engine: &FPEngine,
    fp_config: &Arc<Config<'a>>,
    paths: &[FHIRString],
    input: &'a dyn MetaValue,
) -> Result<Nodes<'a>, OperationOutcomeError> {
    let (mut nodes, mut arenas) = (Vec::new(), Vec::new());
    let mut seen = HashSet::new();
    // The input is walked but is not an item.
    let mut stack = vec![(input, false)];

    while let Some((node, is_item)) = stack.pop() {
        if is_item {
            // Keyed by type too: a struct and its first field share an address.
            if !seen.insert((
                std::ptr::from_ref(node).cast::<()>().addr(),
                node.fhir_type(),
            )) {
                continue;
            }
            if nodes.len() >= REPEAT_ITEM_LIMIT {
                return Err(OperationOutcomeError::error(
                    IssueType::exception(),
                    format!("repeat produced more than {REPEAT_ITEM_LIMIT} items"),
                ));
            }
            nodes.push(node);
        }

        // Reversed so the first child pops first.
        let start = stack.len();
        for path in paths.iter().filter_map(|path| path.value.as_deref()) {
            let found = evaluate(fp_engine, fp_config, path, node, "repeat expression").await?;
            if !found.is_empty() {
                stack.extend(found.values().iter().map(|&child| (child, true)));
                arenas.push(found);
            }
        }
        stack[start..].reverse();
    }

    Ok((nodes, arenas))
}

/// Each select's rows on `node`, in order.
async fn select_rows<'r, 'v: 'r>(
    fp_engine: &'r FPEngine,
    fp_config: &Arc<Config<'r>>,
    selects: &'v [ViewDefinitionSelect],
    node: &'r dyn MetaValue,
) -> Result<Vec<Vec<Row<'v>>>, OperationOutcomeError> {
    try_join_all(
        selects
            .iter()
            .map(|select| process_select(fp_engine, fp_config.clone(), select, node)),
    )
    .await
}

/// A select's rows: per focus node, its columns joined with its nested and `unionAll` rows. The
/// focus nodes are the `forEach`, `forEachOrNull` or `repeat` items, else the input.
fn process_select<'r, 'v: 'r>(
    fp_engine: &'r FPEngine,
    fp_config: Arc<Config<'r>>,
    select: &'v ViewDefinitionSelect,
    input: &'r dyn MetaValue,
) -> BoxFuture<'r, Result<Vec<Row<'v>>, OperationOutcomeError>> {
    Box::pin(async move {
        let for_each = select.forEach.as_ref().and_then(|p| p.value.as_deref());
        let for_each_or_null = select
            .forEachOrNull
            .as_ref()
            .and_then(|p| p.value.as_deref());

        // `_arenas` keeps `nodes` valid until the rows are built.
        let (nodes, _arenas) = match (for_each, for_each_or_null, select.repeat.as_deref()) {
            (Some(path), _, _) => {
                nodes_of(evaluate(fp_engine, &fp_config, path, input, "forEach expression").await?)
            }
            (None, Some(path), _) => nodes_of(
                evaluate(
                    fp_engine,
                    &fp_config,
                    path,
                    input,
                    "forEachOrNull expression",
                )
                .await?,
            ),
            (None, None, Some(paths)) => walk_repeat(fp_engine, &fp_config, paths, input).await?,
            (None, None, None) => (vec![input], Vec::new()),
        };

        if nodes.is_empty() && for_each.is_none() && for_each_or_null.is_some() {
            let mut null_row = Row::new();
            collect_null_columns(select, &mut null_row)?;
            return Ok(vec![null_row]);
        }

        let mut rows = Vec::with_capacity(nodes.len());
        for node in nodes {
            let columns = column_row(fp_engine, &fp_config, select, node).await?;
            let nested = select_rows(
                fp_engine,
                &fp_config,
                select.select.as_deref().unwrap_or_default(),
                node,
            )
            .await?;
            let union = match select.unionAll.as_deref() {
                Some(branches) => Some(
                    select_rows(fp_engine, &fp_config, branches, node)
                        .await?
                        .concat(),
                ),
                None => None,
            };

            rows.extend(
                nested
                    .into_iter()
                    .chain(union)
                    .fold(vec![columns], |rows, more| join(&rows, more)),
            );
        }

        Ok(rows)
    })
}

/// Adds a null for every column of the select, its nested selects and its `unionAll` branches.
fn collect_null_columns<'a>(
    select_statement: &'a ViewDefinitionSelect,
    row: &mut Row<'a>,
) -> Result<(), OperationOutcomeError> {
    for column in select_statement.column.iter().flatten() {
        let Some(name) = column.name.value.as_deref() else {
            return Err(OperationOutcomeError::error(
                IssueType::invalid(),
                "Column name is required".to_string(),
            ));
        };

        row.insert(name, OutputResults::Scalar(None));
    }

    for nested in select_statement
        .select
        .iter()
        .flatten()
        .chain(select_statement.unionAll.iter().flatten())
    {
        collect_null_columns(nested, row)?;
    }

    Ok(())
}

/// A select's own columns on `node`, as one row.
async fn column_row<'r, 'v>(
    fp_engine: &FPEngine,
    fp_config: &Arc<Config<'r>>,
    select: &'v ViewDefinitionSelect,
    node: &'r dyn MetaValue,
) -> Result<Row<'v>, OperationOutcomeError> {
    let mut row = Row::new();

    for column in select.column.iter().flatten() {
        let Some(path) = column.path.value.as_deref() else {
            return Err(OperationOutcomeError::error(
                IssueType::invalid(),
                "Column path is required".to_string(),
            ));
        };

        let Some(name) = column.name.value.as_deref() else {
            return Err(OperationOutcomeError::error(
                IssueType::invalid(),
                "Column name is required".to_string(),
            ));
        };

        let found = evaluate(fp_engine, fp_config, path, node, "expression").await?;
        let values = found.values();

        let column_type = column
            .type_
            .as_ref()
            .and_then(|t| t.value.as_deref())
            .unwrap_or_else(|| values.first().map_or("string", |value| value.fhir_type()));
        let convert = |value: &&dyn MetaValue| {
            conversions::primitives::convert_meta_value(column_type, *value)
        };

        let value = if column.collection.as_ref().and_then(|c| c.value) == Some(true) {
            OutputResults::Collection(values.iter().map(convert).collect::<Result<_, _>>()?)
        } else {
            match values {
                [] => OutputResults::Scalar(None),
                [value] => OutputResults::Scalar(convert(value)?),
                _ => {
                    return Err(OperationOutcomeError::error(
                        IssueType::invalid(),
                        "Column result is a collection but the column is not marked as a collection"
                            .to_string(),
                    ));
                }
            }
        };

        row.insert(name, value);
    }

    Ok(row)
}

/// The resources a view runs over: those of its resource type. A Bundle of another type stands
/// for its entries, so its entries of that type are kept. Resources sent with the request can be
/// of any type, and a view gives rows only for its own.
fn resources_of_type<'r>(
    resources: &'r [Resource],
    resource_type: &ResourceType,
) -> Vec<&'r Resource> {
    let mut matching = Vec::new();
    for resource in resources {
        if &resource.resource_type() == resource_type {
            matching.push(resource);
        } else if let Resource::Bundle(bundle) = resource {
            matching.extend(
                bundle
                    .entry
                    .iter()
                    .flatten()
                    .filter_map(|entry| entry.resource.as_deref())
                    .filter(|resource| &resource.resource_type() == resource_type),
            );
        }
    }

    matching
}

async fn passes_where_clauses(
    fp_engine: &FPEngine,
    variables: Arc<HashMap<String, &dyn MetaValue>>,
    where_clauses: &[&str],
    resource: &Resource,
) -> Result<bool, OperationOutcomeError> {
    for where_clause in where_clauses {
        let result = fp_engine
            .evaluate_with_config(
                where_clause,
                vec![resource],
                Arc::new(Config::builder().with_variable_resolver(
                    haste_fhirpath::ExternalConstantResolver::Variable(variables.clone()),
                )),
            )
            .await
            .map_err(|e| {
                OperationOutcomeError::error(
                    fhirpath_issue_type(&e),
                    format!("Error evaluating where clause expression: {e}"),
                )
            })?;

        // An empty result excludes the resource, like `false`.
        if result.is_empty() {
            return Ok(false);
        }

        let bool_results = result
            .iter()
            .map(|v| match v.fhir_type() {
                "boolean" => Ok(v
                    .as_any()
                    .downcast_ref::<FHIRBoolean>()
                    .and_then(|b| b.value.as_ref())
                    .unwrap_or(&false)),
                "http://hl7.org/fhirpath/System.Boolean" => {
                    Ok(v.as_any().downcast_ref::<bool>().unwrap_or(&false))
                }
                _ => Err(OperationOutcomeError::error(
                    IssueType::invalid(),
                    format!(
                        "Where clause expression must evaluate to a boolean, got: {}",
                        v.fhir_type()
                    ),
                )),
            })
            .collect::<Result<Vec<_>, _>>()?;

        if bool_results.iter().any(|v| !**v) {
            return Ok(false);
        }
    }

    Ok(true)
}

async fn process_view_definition<
    CTX: Send + Sync + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + Send + Sync + 'static,
>(
    context: CTX,
    output_format: &BoundCode<OutputFormatCodes>,
    client: Arc<Client>,
    view_definition: &ViewDefinition,
    input: &ViewDefinitionRun::Input,
    scope: Option<Vec<String>>,
    limits: RunLimits,
) -> Result<Binary, OperationOutcomeError> {
    let fp_engine = FPEngine::new();
    let run = Run {
        fp_engine: &fp_engine,
        variables: Arc::new(build_hashmap_fp_variables(view_definition)),
        view_definition,
        where_clauses: view_definition
            .where_
            .iter()
            .flatten()
            .filter_map(|w| w.path.value.as_deref())
            .collect(),
        resource_type: view_resource_type(view_definition)?,
        limit: requested_limit(input),
        limits,
    };

    let results = view_rows(context, client.as_ref(), &run, input, scope).await?;

    let include_header = input
        .header
        .as_ref()
        .and_then(|header| header.value)
        .unwrap_or(true);

    // `contentType` is required on Binary; without it a client cannot read
    // the result back.
    let (content_type, data) = match output_format {
        binding if binding == &OutputFormatCodes::csv() => {
            ("text/csv", output::csv::csv(&results, include_header)?)
        }
        binding if binding == &OutputFormatCodes::json() => {
            ("application/json", output::json::json(&results)?)
        }
        binding if binding == &OutputFormatCodes::ndjson() => {
            ("application/x-ndjson", output::ndjson::ndjson(results)?)
        }
        _ => {
            return Err(OperationOutcomeError::error(
                IssueType::not_supported(),
                format!("Output format '{output_format:?}' is not supported"),
            ));
        }
    };

    Ok(Binary {
        contentType: Box::new(FHIRCode {
            value: Some(content_type.to_string()),
            ..Default::default()
        }),
        data: Some(Box::new(FHIRBase64Binary {
            value: Some(general_purpose::STANDARD.encode(&data)),
            ..Default::default()
        })),
        ..Default::default()
    })
}

/// Runs a FHIR `ViewDefinition` and returns the generated result in the requested
/// output format.
///
/// The output format is read from the `_format` input parameter. If `_format` is
/// absent or does not contain a recognized [`OutputFormatCodes`] value, ndjson is
/// used, the specification's default.
///
/// A run that would pass `limits` is refused with `too-costly`.
///
/// The view is read through `client`; the `patient` and `group` resources that scope the run
/// through `read_many`, in one batch.
///
/// # Errors
///
/// Returns [`OperationOutcomeError`] if a parameter is refused, the view definition cannot be
/// resolved, the run would pass `limits`, or processing the view definition fails.
pub async fn view_definition_run<
    CTX: Send + Sync + Clone + 'static,
    Client: FHIRClient<CTX, OperationOutcomeError> + Send + Sync + 'static,
>(
    context: CTX,
    client: Arc<Client>,
    read_many: &ReadMany<CTX>,
    input: &ViewDefinitionRun::Input,
    limits: RunLimits,
) -> Result<ViewDefinitionRun::Output, OperationOutcomeError> {
    check_parameters(input, &limits)?;

    let output_format = input
        .format
        .as_ref()
        .and_then(|v| v.value.as_ref())
        .and_then(|s| BoundCode::<OutputFormatCodes>::new(s))
        .unwrap_or(OutputFormatCodes::ndjson());

    // The view and the scope don't depend on each other, so they are looked up at once.
    let (view_definition, scope) = try_join(
        resolve_view_definition(context.clone(), client.as_ref(), input),
        patient_scope(context.clone(), read_many, input),
    )
    .await?;

    let output = process_view_definition(
        context,
        &output_format,
        client,
        &view_definition,
        input,
        scope,
        limits,
    )
    .await?;

    Ok(ViewDefinitionRun::Output { return_: output })
}
