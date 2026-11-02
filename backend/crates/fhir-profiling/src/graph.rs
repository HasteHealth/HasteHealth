//! Assembles the profiles and value sets one validation needs into a
//! [`ProfileGraph`], and caches compiled profiles, value set codes and whole
//! graphs for as long as the resolver keeps handing out the same resources.
//!
//! Every cache is keyed by the identity of the `Arc<Resource>` the resolver
//! returned and holds a `Weak` to it. An entry is current while that `Arc` is
//! still alive somewhere, which for the server means the resolver's own cache
//! still serves it; once the resolver drops a resource (its TTL passed, a
//! package was installed), the entry is stale and the next validation builds
//! it again. The `Weak` also keeps the allocation from being reused, so a key
//! can never match a different resource.

use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, PoisonError, RwLock, Weak},
};

use haste_fhir_client::canonical_resolver::CanonicalResolver;
use haste_fhir_model::r4::generated::{
    resources::{Resource, ResourceType},
    terminology::IssueType,
};
use haste_fhir_operation_error::OperationOutcomeError;

use crate::{
    compile::{
        CompileError, CompiledProfile, DiscriminatorKind, DiscriminatorPath, Node, NodeId, compile,
    },
    slicing::{ValueSetCodes, value_set_codes},
};

/// The closure of profiles one validation can reach, with the discriminator
/// matchers of every slice resolved.
#[derive(Debug)]
pub(crate) struct ProfileGraph {
    /// `profiles[0]` is the profile being validated.
    pub profiles: Vec<GraphProfile>,
    /// Every resource the graph was built from.
    sources: Vec<Weak<Resource>>,
}

#[derive(Debug)]
pub(crate) struct GraphProfile {
    pub compiled: Arc<CompiledProfile>,
    /// What each canonical slot of `compiled` resolved to.
    pub links: Vec<Link>,
    /// One per `(slice, discriminator)` pair, at the slice's `matchers` offset.
    pub matchers: Vec<Matcher>,
}

#[derive(Debug, Clone)]
pub(crate) enum Link {
    Profile(usize),
    /// The resolver does not know the canonical.
    Missing(String),
    NotStructureDefinition,
    NoRootElement,
}

impl Link {
    /// The error validation reports when a value reaches a link that is not a
    /// profile.
    pub(crate) fn error(&self) -> OperationOutcomeError {
        match self {
            Link::Profile(_) => unreachable!("a resolved profile is not an error"),
            Link::Missing(canonical) => OperationOutcomeError::error(
                IssueType::exception(),
                format!("Failed to resolve profile canonical: {canonical}"),
            ),
            Link::NotStructureDefinition => OperationOutcomeError::error(
                IssueType::invalid(),
                "Profile resource must be a StructureDefinition".to_string(),
            ),
            Link::NoRootElement => OperationOutcomeError::error(
                IssueType::exception(),
                "Invalid element path: /snapshot/element/0".to_string(),
            ),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Matcher {
    /// The element whose `fixed[x]`, `pattern[x]`, binding or types decide the
    /// slice: a graph profile and a node in it.
    pub target: Result<(usize, NodeId), TargetError>,
    pub binding: BindingCodes,
}

/// Why a discriminator path has no target element; reported when a value is
/// tested against the slice, as before.
#[derive(Debug, Clone)]
pub(crate) enum TargetError {
    /// No element on the path.
    NotFound,
    /// A profile on the path could not be used.
    Link(Link),
}

/// For a `value` discriminator on a slice defined by its binding alone, the
/// codes that binding allows.
#[derive(Debug)]
pub(crate) enum BindingCodes {
    /// The target has no binding, or has a fixed value or pattern instead.
    None,
    Codes(Arc<ValueSetCodes>),
    /// A mandatory slice whose value set cannot be resolved; reported when a
    /// value is tested against the slice.
    Unresolvable(String),
}

impl ProfileGraph {
    fn is_current(&self) -> bool {
        self.sources.iter().all(|source| source.strong_count() > 0)
    }
}

/// A cache keyed by the address of an `Arc<Resource>`, valid while that `Arc`
/// is alive.
struct ResourceCache<T>(RwLock<HashMap<usize, (Weak<Resource>, T)>>);

/// Above this many entries an insert first drops the stale ones.
const SWEEP_ABOVE: usize = 256;

impl<T: Clone> ResourceCache<T> {
    fn new() -> Self {
        Self(RwLock::new(HashMap::new()))
    }

    fn get(&self, resource: &Arc<Resource>, current: impl Fn(&T) -> bool) -> Option<T> {
        let cache = self.0.read().unwrap_or_else(PoisonError::into_inner);
        cache
            .get(&key(resource))
            .filter(|(source, value)| source.strong_count() > 0 && current(value))
            .map(|(_, value)| value.clone())
    }

    fn insert(&self, resource: &Arc<Resource>, value: T, current: impl Fn(&T) -> bool) {
        let mut cache = self.0.write().unwrap_or_else(PoisonError::into_inner);
        if cache.len() >= SWEEP_ABOVE {
            cache.retain(|_, (source, value)| source.strong_count() > 0 && current(value));
        }
        cache.insert(key(resource), (Arc::downgrade(resource), value));
    }
}

fn key(resource: &Arc<Resource>) -> usize {
    Arc::as_ptr(resource) as usize
}

fn always<T>(_: &T) -> bool {
    true
}

type Compiled = Result<Arc<CompiledProfile>, Link>;

static COMPILED: LazyLock<ResourceCache<Compiled>> = LazyLock::new(ResourceCache::new);
/// A value set's codes and the resources (it and the value sets it includes)
/// they were read from.
#[derive(Debug)]
struct ValueSetEntry {
    codes: Option<Arc<ValueSetCodes>>,
    sources: Vec<Weak<Resource>>,
}

static VALUE_SETS: LazyLock<ResourceCache<Arc<ValueSetEntry>>> = LazyLock::new(ResourceCache::new);
static GRAPHS: LazyLock<ResourceCache<Arc<ProfileGraph>>> = LazyLock::new(ResourceCache::new);

/// The compiled form of `resource`, or why it has none.
fn compiled(resource: &Arc<Resource>) -> Compiled {
    if let Some(compiled) = COMPILED.get(resource, always) {
        return compiled;
    }
    let compiled = match resource.as_ref() {
        Resource::StructureDefinition(sd) => {
            compile(sd).map(Arc::new).map_err(|error| match error {
                CompileError::NoRootElement => Link::NoRootElement,
            })
        }
        _ => Err(Link::NotStructureDefinition),
    };
    COMPILED.insert(resource, compiled.clone(), always);
    compiled
}

/// The graph for `root`, from the cache when the resolver still serves every
/// resource it was built from.
///
/// # Errors
///
/// Returns the resolver's error, or an error when `root` is not a
/// `StructureDefinition` with a snapshot.
pub(crate) async fn graph_for(
    resolver: &impl CanonicalResolver,
    root: &Arc<Resource>,
) -> Result<Arc<ProfileGraph>, OperationOutcomeError> {
    if let Some(graph) = GRAPHS.get(root, |graph| graph.is_current()) {
        return Ok(graph);
    }
    let graph = Arc::new(assemble(resolver, root).await?);
    GRAPHS.insert(root, graph.clone(), |graph| graph.is_current());
    Ok(graph)
}

async fn assemble(
    resolver: &impl CanonicalResolver,
    root: &Arc<Resource>,
) -> Result<ProfileGraph, OperationOutcomeError> {
    let mut sources = vec![Arc::downgrade(root)];
    let mut profiles = vec![GraphProfile {
        compiled: compiled(root).map_err(|link| link.error())?,
        links: Vec::new(),
        matchers: Vec::new(),
    }];
    let mut by_canonical: HashMap<String, Link> = HashMap::new();

    // Every profile reachable through `type.profile`, breadth first. A
    // canonical the resolver does not know, or that is not a usable
    // StructureDefinition, is linked to the error to report if a value reaches
    // it; a branch no value takes never fails a validation.
    let mut next = 0;
    while next < profiles.len() {
        let canonicals = profiles[next].compiled.canonicals.clone();
        let mut links = Vec::with_capacity(canonicals.len());
        for canonical in canonicals {
            let link = if let Some(link) = by_canonical.get(&canonical) {
                link.clone()
            } else {
                let link = link(resolver, &canonical, &mut profiles, &mut sources).await?;
                by_canonical.insert(canonical, link.clone());
                link
            };
            links.push(link);
        }
        profiles[next].links = links;
        next += 1;
    }

    for index in 0..profiles.len() {
        let matchers = matchers(resolver, &profiles, index, &mut sources).await?;
        profiles[index].matchers = matchers;
    }

    Ok(ProfileGraph { profiles, sources })
}

/// Resolves `canonical`, adding its profile to the graph when it has one.
async fn link(
    resolver: &impl CanonicalResolver,
    canonical: &str,
    profiles: &mut Vec<GraphProfile>,
    sources: &mut Vec<Weak<Resource>>,
) -> Result<Link, OperationOutcomeError> {
    let Some(resource) = resolver
        .resolve(ResourceType::StructureDefinition, canonical)
        .await?
    else {
        return Ok(Link::Missing(canonical.to_string()));
    };
    Ok(match compiled(&resource) {
        Ok(compiled) => {
            sources.push(Arc::downgrade(&resource));
            profiles.push(GraphProfile {
                compiled,
                links: Vec::new(),
                matchers: Vec::new(),
            });
            Link::Profile(profiles.len() - 1)
        }
        Err(link) => link,
    })
}

async fn matchers(
    resolver: &impl CanonicalResolver,
    profiles: &[GraphProfile],
    index: usize,
    sources: &mut Vec<Weak<Resource>>,
) -> Result<Vec<Matcher>, OperationOutcomeError> {
    let compiled = &profiles[index].compiled;
    let mut matchers = Vec::with_capacity(compiled.matcher_count);
    for node in &compiled.nodes {
        for slicing in &node.slicings {
            let Ok(discriminators) = &slicing.discriminators else {
                continue;
            };
            for slice in &slicing.slices {
                debug_assert_eq!(matchers.len(), slice.matchers);
                for discriminator in discriminators {
                    let target = match &discriminator.path {
                        DiscriminatorPath::This => Ok((index, slice.node)),
                        DiscriminatorPath::Members(segments) => {
                            find_target(profiles, index, slice.node, segments, 0)
                        }
                        DiscriminatorPath::Expression(expression) => find_target(
                            profiles,
                            index,
                            slice.node,
                            &expression_segments(expression),
                            0,
                        ),
                    };
                    let binding = match target {
                        Ok((profile, node)) if discriminator.kind == DiscriminatorKind::Value => {
                            binding_codes(
                                resolver,
                                &profiles[profile].compiled.nodes[node],
                                compiled.nodes[slice.node].min,
                                sources,
                            )
                            .await?
                        }
                        _ => BindingCodes::None,
                    };
                    matchers.push(Matcher { target, binding });
                }
            }
        }
    }
    Ok(matchers)
}

/// The segments an expression discriminator path would walk, for finding its
/// target element: the `FHIRPath` form is only used for the values.
fn expression_segments(expression: &str) -> Vec<String> {
    expression
        .replace("$this", "")
        .split('.')
        .filter(|segment| !segment.is_empty())
        .map(ToString::to_string)
        .collect()
}

/// Deepest chain of elements and profiles a discriminator path may cross.
const MAX_TARGET_DEPTH: usize = 32;

/// The element `segments` lead to from `node`: at each step a profile the
/// element's type names is searched before the element's own children, as the
/// element path search always has, and a profile that cannot be used stops the
/// search with its error.
fn find_target(
    profiles: &[GraphProfile],
    profile: usize,
    node: NodeId,
    segments: &[String],
    depth: usize,
) -> Result<(usize, NodeId), TargetError> {
    let Some((segment, rest)) = segments.split_first() else {
        return Ok((profile, node));
    };
    if depth > MAX_TARGET_DEPTH {
        return Err(TargetError::NotFound);
    }
    let graph_profile = &profiles[profile];
    let current = &graph_profile.compiled.nodes[node];
    for slot in current
        .types
        .iter()
        .flatten()
        .flat_map(|type_| &type_.profiles)
    {
        match &graph_profile.links[*slot] {
            Link::Profile(nested) => {
                if let Ok(found) = find_target(profiles, *nested, 0, segments, depth + 1) {
                    return Ok(found);
                }
            }
            link => return Err(TargetError::Link(link.clone())),
        }
    }
    current
        .lookup
        .iter()
        .filter(|child| {
            let child = &graph_profile.compiled.nodes[**child];
            child.segment == *segment || child.field == *segment
        })
        .find_map(|child| find_target(profiles, profile, *child, rest, depth + 1).ok())
        .ok_or(TargetError::NotFound)
}

/// Bounds how deeply value sets may include each other, so a cycle ends.
const MAX_VALUE_SET_DEPTH: u8 = 8;

/// For a `value` discriminator on a slice defined by its binding alone (no
/// `fixed[x]` or `pattern[x]`), the codes that binding allows. Only a required
/// binding discriminates; any other allows none. A value set that cannot be
/// resolved allows none, or is reported when the slice is mandatory.
async fn binding_codes(
    resolver: &impl CanonicalResolver,
    target: &Node,
    slice_min: u64,
    sources: &mut Vec<Weak<Resource>>,
) -> Result<BindingCodes, OperationOutcomeError> {
    let Some(binding) = target
        .binding
        .as_ref()
        .filter(|_| target.fixed.is_none() && target.pattern.is_none())
    else {
        return Ok(BindingCodes::None);
    };
    let Some(url) = binding.value_set.as_deref().filter(|_| binding.required) else {
        return Ok(BindingCodes::Codes(Arc::new(ValueSetCodes::default())));
    };
    let codes = match resolver.resolve(ResourceType::ValueSet, url).await? {
        None => None,
        Some(resource) => {
            let entry = if let Some(entry) = VALUE_SETS.get(&resource, always) {
                entry
            } else {
                let mut read = Vec::new();
                let codes = value_set_codes(resolver, url, MAX_VALUE_SET_DEPTH, &mut read)
                    .await?
                    .map(Arc::new);
                let entry = Arc::new(ValueSetEntry {
                    codes,
                    sources: read,
                });
                VALUE_SETS.insert(&resource, entry.clone(), always);
                entry
            };
            sources.push(Arc::downgrade(&resource));
            sources.extend(entry.sources.iter().cloned());
            entry.codes.clone()
        }
    };
    Ok(match codes {
        Some(codes) => BindingCodes::Codes(codes),
        None if slice_min == 0 => BindingCodes::Codes(Arc::new(ValueSetCodes::default())),
        None => BindingCodes::Unresolvable(url.to_string()),
    })
}
