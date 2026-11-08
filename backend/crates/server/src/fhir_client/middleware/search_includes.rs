//! `_include` and `_revinclude`: the resources that travel with a page of
//! matches.
//!
//! `_include` follows references outward: the match holds the reference, so
//! the target is read by id. `_revinclude` follows them inward: another
//! resource references the match, so the referrers are searched for.
//!
//! Both are result parameters, not filters, so they are resolved here from the
//! page the backend returned rather than folded into the match query, where
//! they would dilute the `_count` page with extras. That also bounds the work
//! by the page: `_include` reads at most `matches × parameters` distinct
//! `(type, id)` pairs by primary key, and `_revinclude` runs one search per
//! directive per hop with every page id as the value, answered from the
//! reference index and confined to the caller's compartment.
//!
//! [`MAX_INCLUDED`], [`MAX_REVINCLUDE_PER_DIRECTIVE`] and [`MAX_ITERATE_DEPTH`]
//! bound a pathological request. All truncate rather than fail: a partial set
//! of includes is more useful than an error.

use std::collections::{HashMap, HashSet, hash_map::Entry};
use std::sync::LazyLock;

use haste_fhir_client::{
    request::{FHIRSearchTypeRequest, SearchRequest},
    url::{Parameter, ParsedParameter, ParsedParameters},
};
use haste_fhir_model::r4::generated::resources::{Resource, ResourceType};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::{
    ResolvedParameter, SearchEngine, SearchOptions, SearchParameterResolve, indexing_conversion,
    memory::R4_SEARCH_PARAMETERS_INDEX,
};
use haste_fhirpath::FPEngine;
use haste_jwt::{ResourceId, VersionId};
use haste_repository::fhir::{CachePolicy, FHIRRepository};

use crate::fhir_client::{
    ClientState, ServerCTX,
    middleware::{Backend, auth_z::patient_scope},
};

/// Ceiling on resources one search may add through `_include` and
/// `_revinclude` together; large pages with `:iterate` are what it is for.
const MAX_INCLUDED: usize = 1_000;

/// Ceiling on referrers one `_revinclude` directive may return per hop, so a
/// patient with 200 000 Observations cannot flood the Bundle. Past it, a
/// search of that type with its own paging is the answer.
const MAX_REVINCLUDE_PER_DIRECTIVE: u64 = 500;

/// How many hops `:iterate` may take. FHIR leaves the depth to the server; two
/// covers Encounter → Patient → Organization without walking a dense graph.
const MAX_ITERATE_DEPTH: usize = 2;

/// Constructing an engine is not free, and it holds no per-request state.
static FP_ENGINE: LazyLock<FPEngine> = LazyLock::new(FPEngine::new);

/// `*` in an `_include` value: every type, or every reference parameter.
const WILDCARD: &str = "*";

/// One parsed `_include` or `_revinclude` value. `Observation:subject:Patient`
/// parses to source `Observation`, parameter `subject`, target `Patient`. Read
/// outward for `_include` and inward for `_revinclude`, where the source is
/// the referrer's type and the target restricts which matched type it points
/// at.
#[derive(Debug, Clone, PartialEq, Eq)]
struct IncludeDirective {
    /// The type whose parameter is followed; [`WILDCARD`] for any matched type.
    source_type: String,
    /// The reference search parameter to follow; [`WILDCARD`] for all of them.
    parameter: String,
    /// Limits the targets to one type, for a parameter that can point at
    /// several.
    target_type: Option<String>,
    /// `:iterate`: also apply this directive to resources already added.
    iterate: bool,
}

/// A reference to resolve: a type and an id within this project.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct IncludeTarget {
    resource_type: ResourceType,
    id: String,
}

/// Expressions per (directive index, source type). `:iterate` hops repeat the
/// same lookups, so each pair is resolved once per search.
type ExpressionCache = HashMap<(usize, ResourceType), Vec<String>>;

/// Everything the `_include` and `_revinclude` parameters add to one page of
/// matches.
///
/// Hop 0 runs every directive against the matches. Up to [`MAX_ITERATE_DEPTH`]
/// further hops run only the `:iterate` directives, in both directions,
/// against what the previous hop added. Nothing in the page or already added
/// is added twice.
pub async fn resolve<B: Backend>(
    state: &ClientState<B>,
    ctx: &ServerCTX<B>,
    parameters: &ParsedParameters,
    matches: &[Resource],
) -> Result<Vec<Resource>, OperationOutcomeError> {
    let includes = directives_named(parameters, "_include");
    let revincludes = directives_named(parameters, "_revinclude");
    if (includes.is_empty() && revincludes.is_empty()) || matches.is_empty() {
        return Ok(Vec::new());
    }

    // Matches are already in the Bundle.
    let mut emitted: HashSet<IncludeTarget> = matches
        .iter()
        .filter_map(|m| {
            Some(IncludeTarget {
                resource_type: m.resource_type(),
                id: m.id().clone()?,
            })
        })
        .collect();
    let mut cache = ExpressionCache::new();
    let mut added: Vec<Resource> = Vec::new();
    let iterates = includes.iter().chain(&revincludes).any(|d| d.iterate);
    // Where the previous hop's additions start in `added`.
    let mut from = 0;

    for hop in 0..=MAX_ITERATE_DEPTH {
        let iterate_only = hop > 0;
        if iterate_only && (!iterates || from == added.len() || added.len() >= MAX_INCLUDED) {
            break;
        }
        let frontier = if iterate_only {
            &added[from..]
        } else {
            matches
        };
        from = added.len();
        let room = MAX_INCLUDED.saturating_sub(added.len());

        let targets = collect_targets(
            ctx,
            frontier,
            &includes,
            iterate_only,
            &mut cache,
            &mut emitted,
            room,
        )
        .await?;
        let referrers = find_referrers(
            state,
            ctx,
            frontier,
            &revincludes,
            iterate_only,
            &mut emitted,
            room.saturating_sub(targets.len()),
        )
        .await?;

        added.extend(read_targets(state, ctx, &targets).await);
        added.extend(read_versions(state, ctx, &referrers).await);
    }

    if added.len() >= MAX_INCLUDED {
        tracing::info!(
            "includes reached the {MAX_INCLUDED}-resource ceiling; \
             the page is returned with those resolved so far"
        );
    }

    Ok(added)
}

/// The directives of the `_include` or `_revinclude` parameter `name`, in the
/// order given.
fn directives_named(parameters: &ParsedParameters, name: &str) -> Vec<IncludeDirective> {
    parameters
        .parameters()
        .iter()
        .filter_map(|parameter| match parameter {
            ParsedParameter::Result(p) | ParsedParameter::Resource(p) => {
                (p.name == name).then_some(p)
            }
        })
        .flat_map(|p| {
            // `:iterate` (or its predecessor `:recurse`) modifies the
            // parameter, so it applies to each of its values.
            let iterate = matches!(p.modifier.as_deref(), Some("iterate" | "recurse"));
            p.value
                .iter()
                .filter_map(move |value| parse_include_value(value, iterate))
        })
        .collect()
}

/// Parses `Type:param`, `Type:param:TargetType`, `Type:*` or `*`. Wildcards
/// fan out to every reference parameter of the matched types, bounded by
/// [`MAX_INCLUDED`] and [`MAX_ITERATE_DEPTH`].
fn parse_include_value(value: &str, iterate: bool) -> Option<IncludeDirective> {
    let parts: Vec<&str> = value.split(':').collect();
    let (source_type, parameter, target_type) = match parts[..] {
        [WILDCARD] => (WILDCARD, WILDCARD, None),
        [source_type, parameter] => (source_type, parameter, None),
        [source_type, parameter, target_type] => (source_type, parameter, Some(target_type)),
        _ => return None,
    };
    if source_type.is_empty() || parameter.is_empty() || target_type == Some("") {
        return None;
    }

    Some(IncludeDirective {
        source_type: source_type.to_string(),
        parameter: parameter.to_string(),
        target_type: target_type.map(str::to_string),
        iterate,
    })
}

/// The references `directives` (only the `:iterate` ones when `iterate_only`)
/// select out of `frontier`, by evaluating each directive's expressions on
/// each resource of its type. Returns at most `room` targets not yet emitted,
/// and marks them emitted.
async fn collect_targets<B: Backend>(
    ctx: &ServerCTX<B>,
    frontier: &[Resource],
    directives: &[IncludeDirective],
    iterate_only: bool,
    cache: &mut ExpressionCache,
    emitted: &mut HashSet<IncludeTarget>,
    room: usize,
) -> Result<Vec<IncludeTarget>, OperationOutcomeError> {
    let mut targets = Vec::new();

    for (index, directive) in directives.iter().enumerate() {
        if iterate_only && !directive.iterate {
            continue;
        }

        for resource in frontier {
            let resource_type = resource.resource_type();
            if directive.source_type != WILDCARD && resource_type.as_ref() != directive.source_type
            {
                continue;
            }

            let expressions: &[String] = match cache.entry((index, resource_type)) {
                Entry::Occupied(entry) => entry.into_mut(),
                Entry::Vacant(entry) => {
                    let expressions = directive_expressions(ctx, directive, &entry.key().1).await?;
                    entry.insert(expressions)
                }
            };
            for expression in expressions {
                collect_from(resource, expression, directive, &mut targets, emitted, room).await;
            }
        }
    }

    Ok(targets)
}

/// Evaluates `expression` on `resource` and pushes each reference that passes
/// `directive`'s target type and is not yet emitted, while `targets` has room.
async fn collect_from(
    resource: &Resource,
    expression: &str,
    directive: &IncludeDirective,
    targets: &mut Vec<IncludeTarget>,
    emitted: &mut HashSet<IncludeTarget>,
    room: usize,
) {
    // An expression that fails on this resource does not fail the search.
    let values = match FP_ENGINE.evaluate(expression, vec![resource]).await {
        Ok(values) => values,
        Err(e) => {
            tracing::warn!(
                "_include: failed to evaluate '{expression}' for {}:{}: {e}",
                directive.source_type,
                directive.parameter,
            );
            return;
        }
    };

    for value in &values {
        for reference in indexing_conversion::index_reference_values(value) {
            // A canonical or absolute URL is not a row in this project.
            let (Some(resource_type), Some(id)) = (reference.resource_type(), reference.id())
            else {
                continue;
            };
            if directive
                .target_type
                .as_deref()
                .is_some_and(|wanted| wanted != resource_type)
            {
                continue;
            }
            // A type this server does not store cannot be read.
            let Ok(resource_type) = ResourceType::try_from(resource_type) else {
                continue;
            };

            let target = IncludeTarget {
                resource_type,
                id: id.to_string(),
            };
            if targets.len() < room && emitted.insert(target.clone()) {
                targets.push(target);
            }
        }
    }
}

/// The expressions `directive` follows on `resource_type`: one for a named
/// parameter, every reference parameter's for a wildcard.
async fn directive_expressions<B: Backend>(
    ctx: &ServerCTX<B>,
    directive: &IncludeDirective,
    resource_type: &ResourceType,
) -> Result<Vec<String>, OperationOutcomeError> {
    if directive.parameter != WILDCARD {
        let expression = reference_parameter(ctx, directive).await?;
        return Ok(expression
            .map(|(_, expression)| expression)
            .into_iter()
            .collect());
    }

    let mut expressions: Vec<String> = R4_SEARCH_PARAMETERS_INDEX
        .by_resource_type(&ctx.tenant, &ctx.project, resource_type)
        .await?
        .iter()
        .filter_map(reference_expression)
        .collect();
    // Parameters such as `patient` and `subject` share an expression.
    expressions.sort_unstable();
    expressions.dedup();

    Ok(expressions)
}

/// The type and expression of `directive`'s parameter, or `None` when it names
/// no reference parameter of a known type. Not an error: the remaining
/// includes still stand.
async fn reference_parameter<B: Backend>(
    ctx: &ServerCTX<B>,
    directive: &IncludeDirective,
) -> Result<Option<(ResourceType, String)>, OperationOutcomeError> {
    let parameter = match ResourceType::try_from(directive.source_type.as_str()) {
        Ok(resource_type) => R4_SEARCH_PARAMETERS_INDEX
            .by_name(
                &ctx.tenant,
                &ctx.project,
                Some(&resource_type),
                &directive.parameter,
            )
            .await?
            .as_ref()
            .and_then(reference_expression)
            .map(|expression| (resource_type, expression)),
        Err(_) => None,
    };
    if parameter.is_none() {
        tracing::debug!(
            "'{}:{}' is not a reference search parameter",
            directive.source_type,
            directive.parameter
        );
    }
    Ok(parameter)
}

/// The expression of `parameter`, when it is a reference parameter.
fn reference_expression(parameter: &ResolvedParameter) -> Option<String> {
    let parameter = &parameter.search_parameter;
    if parameter.type_.as_str() != Some("reference") {
        return None;
    }
    parameter.expression.as_ref()?.value.clone()
}

/// The type a `_revinclude` directive searches for referrers in, or `None` for
/// an unknown type or parameter, a non-reference parameter, or a wildcard.
/// Wildcards are not followed inward: `_revinclude=Observation:*` would be one
/// search per reference parameter of the type, a cost the client has not
/// bounded.
async fn referring_type<B: Backend>(
    ctx: &ServerCTX<B>,
    directive: &IncludeDirective,
) -> Result<Option<ResourceType>, OperationOutcomeError> {
    if directive.source_type == WILDCARD || directive.parameter == WILDCARD {
        tracing::debug!(
            "_revinclude does not follow wildcards: '{}:{}'",
            directive.source_type,
            directive.parameter
        );
        return Ok(None);
    }
    let parameter = reference_parameter(ctx, directive).await?;
    Ok(parameter.map(|(resource_type, _)| resource_type))
}

/// The `Type/id` of each source a directive's referrers may point at: every
/// source, or only those of the directive's target type.
fn source_ids(sources: &[Resource], directive: &IncludeDirective) -> Vec<String> {
    sources
        .iter()
        .filter(|resource| {
            directive
                .target_type
                .as_deref()
                .is_none_or(|wanted| resource.resource_type().as_ref() == wanted)
        })
        .filter_map(|resource| {
            Some(format!(
                "{}/{}",
                resource.resource_type().as_ref(),
                resource.id().as_deref()?
            ))
        })
        .collect()
}

/// The referrers `directives` (only the `:iterate` ones when `iterate_only`)
/// find for `sources`: one search per directive, with every source id as the
/// parameter's values, confined to the caller's compartment. Returns at most
/// `room` versions to read, skipping what is already emitted and marking the
/// rest emitted.
async fn find_referrers<B: Backend>(
    state: &ClientState<B>,
    ctx: &ServerCTX<B>,
    sources: &[Resource],
    directives: &[IncludeDirective],
    iterate_only: bool,
    emitted: &mut HashSet<IncludeTarget>,
    room: usize,
) -> Result<Vec<VersionId>, OperationOutcomeError> {
    let mut found: Vec<VersionId> = Vec::new();

    for directive in directives {
        if iterate_only && !directive.iterate {
            continue;
        }
        let room = room.saturating_sub(found.len());
        if room == 0 {
            break;
        }

        let Some(resource_type) = referring_type(ctx, directive).await? else {
            continue;
        };
        let ids = source_ids(sources, directive);
        if ids.is_empty() {
            continue;
        }

        // `Type/id` values, so a Group and a Patient sharing an id are not
        // confused.
        let request = SearchRequest::Type(FHIRSearchTypeRequest {
            resource_type: resource_type.clone(),
            parameters: ParsedParameters::new(vec![ParsedParameter::from(Parameter {
                name: directive.parameter.clone(),
                value: ids,
                modifier: None,
                chains: Vec::new(),
            })]),
        });
        let options = SearchOptions {
            count_limit: Some(MAX_REVINCLUDE_PER_DIRECTIVE),
            any_of: ctx.compartment.as_deref().and_then(|patient_id| {
                patient_scope::search_filter(patient_id, Some(&resource_type))
            }),
        };

        let result = match state
            .search
            .search(
                &ctx.fhir_version,
                &ctx.tenant,
                &ctx.project,
                &request,
                Some(options),
            )
            .await
        {
            Ok(result) => result,
            // The backend's failure, not the client's: the matches and the
            // other directives are still worth returning.
            Err(e) => {
                tracing::warn!(
                    "_revinclude '{}:{}' could not be searched: {e}",
                    directive.source_type,
                    directive.parameter,
                );
                continue;
            }
        };

        if result.entries.len() as u64 >= MAX_REVINCLUDE_PER_DIRECTIVE {
            tracing::info!(
                "_revinclude '{}:{}' reached its {MAX_REVINCLUDE_PER_DIRECTIVE}-resource \
                 ceiling; a search of that type pages through the rest",
                directive.source_type,
                directive.parameter,
            );
        }

        found.extend(
            result
                .entries
                .into_iter()
                .filter(|entry| {
                    emitted.insert(IncludeTarget {
                        resource_type: entry.resource_type.clone(),
                        id: entry.id.as_ref().to_string(),
                    })
                })
                .take(room)
                .map(|entry| entry.version_id),
        );
    }

    Ok(found)
}

/// Reads `targets` in one round trip. Deleted or unreadable targets are
/// dropped rather than failing the search: one dangling reference must not
/// hide every match.
async fn read_targets<B: Backend>(
    state: &ClientState<B>,
    ctx: &ServerCTX<B>,
    targets: &[IncludeTarget],
) -> Vec<Resource> {
    let keys: Vec<(ResourceType, ResourceId)> = targets
        .iter()
        .map(|target| {
            (
                target.resource_type.clone(),
                ResourceId::new(target.id.clone()),
            )
        })
        .collect();

    match state
        .repo
        .read_latest_many(&ctx.tenant, &ctx.project, &keys)
        .await
    {
        Ok(resources) => {
            if resources.len() < targets.len() {
                tracing::debug!(
                    "_include: {} of {} targets do not resolve",
                    targets.len() - resources.len(),
                    targets.len()
                );
            }
            resources
        }
        Err(e) => {
            tracing::warn!("_include: {} targets could not be read: {e}", targets.len());
            Vec::new()
        }
    }
}

/// Reads referrers by version, as the matches are read. Unreadable ones are
/// dropped rather than failing the search.
async fn read_versions<B: Backend>(
    state: &ClientState<B>,
    ctx: &ServerCTX<B>,
    version_ids: &[VersionId],
) -> Vec<Resource> {
    if version_ids.is_empty() {
        return Vec::new();
    }
    let version_ids: Vec<&VersionId> = version_ids.iter().collect();

    match state
        .repo
        .read_by_version_ids(
            &ctx.tenant,
            &ctx.project,
            &version_ids,
            CachePolicy::NoCache,
        )
        .await
    {
        Ok(resources) => resources,
        Err(e) => {
            tracing::warn!(
                "_revinclude: {} referrers could not be read: {e}",
                version_ids.len()
            );
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_wildcards() {
        let all = parse_include_value("*", true).unwrap();
        assert_eq!(
            (all.source_type.as_str(), all.parameter.as_str()),
            ("*", "*")
        );
        assert!(all.iterate);

        let typed = parse_include_value("Observation:*", false).unwrap();
        assert_eq!(typed.source_type, "Observation");
        assert_eq!(typed.parameter, "*");
    }
}
