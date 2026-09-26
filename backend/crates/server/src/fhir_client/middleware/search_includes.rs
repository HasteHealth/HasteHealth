//! `_include`: adds the resources a page of matches points at.
//!
//! # Why this lives here and not in the search backend
//!
//! `_include` is a result parameter, not a filter: it never changes which
//! resources match, only what else travels with them. The references it follows
//! are on the matched resources themselves, so once the backend has returned a
//! page there is nothing left to search — the work is reading a bounded set of
//! known ids.
//!
//! # Why this scales
//!
//! Cost is bounded by the page, not the corpus. A page of `_count` matches (50
//! by default, 10 000 at the ceiling) yields at most
//! `matches × include-parameters` references; those are deduplicated to distinct
//! `(type, id)` pairs and read by primary key. A tenant with a hundred million
//! Observations and one with a thousand do the same work for the same page, and
//! no query here scans a table.
//!
//! Two limits keep a pathological request bounded: [`MAX_INCLUDED`] caps how
//! many resources one search may add, and [`MAX_ITERATE_DEPTH`] caps
//! `:iterate` recursion. Both truncate rather than fail, because a chart view
//! that renders without three of four references is more useful than an error.

use std::collections::HashSet;
use std::sync::LazyLock;

use haste_fhir_client::url::{Parameter, ParsedParameter, ParsedParameters};
use haste_fhir_model::r4::generated::resources::{Resource, ResourceType};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::{
    SearchParameterResolve, indexing_conversion, memory::R4_SEARCH_PARAMETERS_INDEX,
};
use haste_fhirpath::FPEngine;
use haste_jwt::{ProjectId, ResourceId, TenantId};
use haste_repository::fhir::FHIRRepository;

/// Ceiling on resources one search may add through `_include`. A page of 50
/// matches with several reference parameters each stays far below it; a
/// 10 000-row page with `:iterate` is what it exists for.
const MAX_INCLUDED: usize = 1_000;

/// How many times `:iterate` may follow references outward. FHIR leaves the
/// depth to the server; two hops covers the shapes clients actually ask for
/// (Encounter → Patient → Organization) without risking a walk across a
/// densely linked graph.
const MAX_ITERATE_DEPTH: usize = 2;

/// Shared with the profiling engine's precedent: constructing one is not free
/// and it holds no per-request state.
static FP_ENGINE: LazyLock<FPEngine> = LazyLock::new(FPEngine::new);

/// One parsed `_include` (or `_revinclude`) value.
///
/// `_include=Observation:subject:Patient` is
/// `source_type = Observation`, `parameter = subject`,
/// `target_type = Some(Patient)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncludeDirective {
    /// The type whose parameter is followed. `*` is not accepted; see
    /// [`parse_include_value`].
    pub source_type: String,
    /// The reference search parameter to follow.
    pub parameter: String,
    /// Restricts which target type is included, for a parameter that can point
    /// at several.
    pub target_type: Option<String>,
    /// `:iterate` — follow this directive against resources already included,
    /// not only against the matches.
    pub iterate: bool,
}

/// The `_include` directives in a search's parameters, in the order given.
#[must_use]
pub fn include_directives(parameters: &ParsedParameters) -> Vec<IncludeDirective> {
    directives_named(parameters, "_include")
}

/// Shared by `_include` and `_revinclude`, which differ only in direction.
pub(crate) fn directives_named(parameters: &ParsedParameters, name: &str) -> Vec<IncludeDirective> {
    parameters
        .parameters()
        .iter()
        .filter_map(|parameter| match parameter {
            ParsedParameter::Result(p) | ParsedParameter::Resource(p) => {
                (p.name == name).then_some(p)
            }
        })
        .flat_map(parsed_values)
        .collect()
}

/// Every directive one parameter carries. `:iterate` (and its R4 predecessor
/// `:recurse`) is a modifier on the parameter, so it applies to each of its
/// values.
fn parsed_values(parameter: &Parameter) -> Vec<IncludeDirective> {
    let iterate = matches!(parameter.modifier.as_deref(), Some("iterate" | "recurse"));

    parameter
        .value
        .iter()
        .filter_map(|value| parse_include_value(value, iterate))
        .collect()
}

/// `Type:param`, `Type:param:TargetType`, or `*`.
///
/// `*` is rejected: including every reference of every match is the one form of
/// this parameter whose cost is not bounded by anything the client stated, and
/// silently ignoring it would hide that. A client that wants everything names
/// the parameters.
fn parse_include_value(value: &str, iterate: bool) -> Option<IncludeDirective> {
    let parts: Vec<&str> = value.split(':').collect();

    match parts[..] {
        [source_type, parameter] if !source_type.is_empty() && !parameter.is_empty() => {
            Some(IncludeDirective {
                source_type: source_type.to_string(),
                parameter: parameter.to_string(),
                target_type: None,
                iterate,
            })
        }
        [source_type, parameter, target_type]
            if !source_type.is_empty() && !parameter.is_empty() && !target_type.is_empty() =>
        {
            Some(IncludeDirective {
                source_type: source_type.to_string(),
                parameter: parameter.to_string(),
                target_type: Some(target_type.to_string()),
                iterate,
            })
        }
        _ => None,
    }
}

/// A reference to resolve: a type and an id within this project.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct IncludeTarget {
    resource_type: String,
    id: String,
}

/// The references `directives` select out of `resources`.
///
/// Evaluates each directive's search parameter expression against each resource
/// of the matching type. A reference with no type (a bare id) or an absolute URL
/// is skipped: neither can be resolved to a row in this project without
/// guessing.
async fn collect_targets(
    tenant: &TenantId,
    project: &ProjectId,
    resources: &[&Resource],
    directives: &[IncludeDirective],
) -> Result<Vec<IncludeTarget>, OperationOutcomeError> {
    let mut targets = Vec::new();
    let mut seen = HashSet::new();

    // One evaluation per (directive, resource of that type).
    for directive in directives {
        let Some(expression) = reference_expression(tenant, project, directive).await? else {
            continue;
        };

        for resource in resources
            .iter()
            .filter(|r| r.resource_type().as_ref() == directive.source_type)
        {
            // A parameter whose expression does not apply to this resource is
            // not a failure of the search: the other includes still stand.
            let evaluated = match FP_ENGINE.evaluate(&expression, vec![*resource]).await {
                Ok(evaluated) => evaluated,
                Err(e) => {
                    tracing::warn!(
                        "_include: failed to evaluate '{expression}' for {}:{}: {e}",
                        directive.source_type,
                        directive.parameter,
                    );
                    continue;
                }
            };

            for value in evaluated.iter() {
                for reference in indexing_conversion::index_reference_values(value) {
                    let (Some(resource_type), Some(id)) =
                        (reference.resource_type(), reference.id())
                    else {
                        // A canonical or absolute URL; not a row in this project.
                        continue;
                    };

                    if let Some(wanted) = directive.target_type.as_deref() {
                        if wanted != resource_type {
                            continue;
                        }
                    }

                    let target = IncludeTarget {
                        resource_type: resource_type.to_string(),
                        id: id.to_string(),
                    };
                    if seen.insert(target.clone()) {
                        targets.push(target);
                    }
                }
            }
        }
    }

    Ok(targets)
}

/// The FHIRPath expression of `directive`'s parameter, or `None` when it names
/// no reference parameter of that type.
///
/// A non-reference parameter is not an error the search should fail on: the
/// client asked for something that cannot be followed, and the remaining
/// includes are still worth returning.
async fn reference_expression(
    tenant: &TenantId,
    project: &ProjectId,
    directive: &IncludeDirective,
) -> Result<Option<String>, OperationOutcomeError> {
    let Ok(resource_type) = ResourceType::try_from(directive.source_type.as_str()) else {
        tracing::debug!(
            "_include names unknown resource type '{}'",
            directive.source_type
        );
        return Ok(None);
    };

    let resolved = R4_SEARCH_PARAMETERS_INDEX
        .by_name(tenant, project, Some(&resource_type), &directive.parameter)
        .await?;

    let Some(resolved) = resolved else {
        tracing::debug!(
            "_include names unknown parameter '{}:{}'",
            directive.source_type,
            directive.parameter
        );
        return Ok(None);
    };

    let search_parameter = &resolved.search_parameter;
    if search_parameter.type_.as_str() != Some("reference") {
        tracing::debug!(
            "_include names non-reference parameter '{}:{}'",
            directive.source_type,
            directive.parameter
        );
        return Ok(None);
    }

    Ok(search_parameter
        .expression
        .as_ref()
        .and_then(|e| e.value.clone()))
}

/// Reads `targets` from the repository, skipping any that no longer resolve.
///
/// Each read is by `(type, id)` primary key. A reference to a deleted or
/// unreadable resource is dropped: an include is supplementary, and failing the
/// whole search over a dangling pointer would make one bad reference hide every
/// match.
async fn read_targets<Repo: FHIRRepository>(
    repo: &Repo,
    tenant: &TenantId,
    project: &ProjectId,
    targets: &[IncludeTarget],
) -> Vec<Resource> {
    let mut resources = Vec::with_capacity(targets.len());

    for target in targets {
        let Ok(resource_type) = ResourceType::try_from(target.resource_type.as_str()) else {
            continue;
        };

        match repo
            .read_latest(
                tenant,
                project,
                &resource_type,
                &ResourceId::new(target.id.clone()),
            )
            .await
        {
            Ok(Some(resource)) => resources.push(resource),
            Ok(None) => {
                tracing::debug!(
                    "_include target {}/{} does not resolve",
                    target.resource_type,
                    target.id
                );
            }
            Err(e) => {
                tracing::warn!(
                    "_include target {}/{} could not be read: {e}",
                    target.resource_type,
                    target.id
                );
            }
        }
    }

    resources
}

/// Everything `_include` adds to one page of matches.
///
/// Runs the non-`:iterate` directives against the matches, then the `:iterate`
/// ones repeatedly against what has been added, up to
/// [`MAX_ITERATE_DEPTH`]. Resources already in the page, and resources already
/// included, are never added twice.
pub async fn resolve_includes<Repo: FHIRRepository>(
    repo: &Repo,
    tenant: &TenantId,
    project: &ProjectId,
    matches: &[Resource],
    directives: Vec<IncludeDirective>,
) -> Result<Vec<Resource>, OperationOutcomeError> {
    if directives.is_empty() || matches.is_empty() {
        return Ok(Vec::new());
    }

    // Matches are already in the Bundle; including one again would duplicate it.
    let mut emitted: HashSet<IncludeTarget> = matches.iter().filter_map(resource_target).collect();

    // direct includes are those without the `:iterate` modifier. They are processed first.
    // Iterate depends on the results of the direct includes. It is processed in subsequent hops.
    let (direct, iterate): (Vec<_>, Vec<_>) = directives.into_iter().partition(|d| !d.iterate);

    let mut included: Vec<Resource> = Vec::new();

    // First hop: the matches.
    let mut frontier = {
        let sources: Vec<&Resource> = matches.iter().collect();
        let targets = collect_targets(tenant, project, &sources, &direct).await?;
        take_new(targets, &mut emitted, included.len())
    };
    included.extend(read_targets(repo, tenant, project, &frontier).await);

    // Further hops: only `:iterate` directives, only against what was added.
    // Brute force over iterate directives up to the maximum depth.
    // For each successive run we use the output from the previous run as the new sources.
    for _ in 0..MAX_ITERATE_DEPTH {
        if iterate.is_empty() || frontier.is_empty() || included.len() >= MAX_INCLUDED {
            break;
        }

        // The resources read for the previous frontier are the tail of
        // `included`; iterating from anything earlier would redo settled work.
        let from = included.len().saturating_sub(frontier.len());
        let sources: Vec<&Resource> = included[from..].iter().collect();

        let targets = collect_targets(tenant, project, &sources, &iterate).await?;
        frontier = take_new(targets, &mut emitted, included.len());
        included.extend(read_targets(repo, tenant, project, &frontier).await);
    }

    if included.len() >= MAX_INCLUDED {
        tracing::info!(
            "_include reached the {MAX_INCLUDED}-resource ceiling; \
             the page is returned with the includes resolved so far"
        );
    }

    Ok(included)
}

/// The targets of `candidates` not yet emitted, capped so the total stays
/// within [`MAX_INCLUDED`]. Records what it returns as emitted.
fn take_new(
    candidates: Vec<IncludeTarget>,
    emitted: &mut HashSet<IncludeTarget>,
    already: usize,
) -> Vec<IncludeTarget> {
    let room = MAX_INCLUDED.saturating_sub(already);

    candidates
        .into_iter()
        .filter(|target| emitted.insert(target.clone()))
        .take(room)
        .collect()
}

fn resource_target(resource: &Resource) -> Option<IncludeTarget> {
    Some(IncludeTarget {
        resource_type: resource.resource_type().as_ref().to_string(),
        id: resource.id().clone()?,
    })
}
