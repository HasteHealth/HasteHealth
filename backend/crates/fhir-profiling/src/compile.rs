//! Compiles a `StructureDefinition` snapshot into a tree the validator walks
//! directly.
//!
//! The snapshot is a flat list of `ElementDefinition`s whose nesting is encoded
//! in their dotted paths. Walking it per resource means re-deriving the
//! parent/child structure, the slicing groups and the field names for every
//! value visited. Compiling once moves all of that to a single pass over the
//! list; the result is self-contained (it clones the constraint values it
//! needs) so it can be cached and shared between requests.

use haste_codegen::utilities::extract::field_name;
use haste_fhir_model::r4::generated::{
    resources::StructureDefinition,
    terminology::{BindingStrength, DiscriminatorType},
    types::{
        ElementDefinition, ElementDefinitionFixedTypeChoice, ElementDefinitionPatternTypeChoice,
        ElementDefinitionSlicingDiscriminator,
    },
};

/// Index into [`CompiledProfile::nodes`].
pub(crate) type NodeId = usize;

/// Index into [`CompiledProfile::canonicals`]: one `type.profile` canonical the
/// profile refers to, numbered so the graph links it in O(1).
pub(crate) type CanonicalSlot = usize;

#[derive(Debug)]
pub(crate) struct CompiledProfile {
    /// The `StructureDefinition.id`, as the diagnostics name it.
    pub id: String,
    /// `nodes[0]` is the root element.
    pub nodes: Vec<Node>,
    /// Canonical URLs by slot, in order of first use.
    pub canonicals: Vec<String>,
    /// How many `(slice, discriminator)` pairs the profile has; the graph
    /// resolves one matcher per pair.
    pub matcher_count: usize,
}

#[derive(Debug)]
pub(crate) struct Node {
    /// The `ElementDefinition.id`, as the diagnostics name it.
    pub element_id: String,
    /// The field the element occupies on an instance (`value` for `value[x]`).
    pub field: String,
    /// The last segment of the element path as written (`value[x]`), which
    /// discriminator paths are matched against.
    pub segment: String,
    pub min: u64,
    pub max: MaxCardinality,
    /// `None` when the element lists no types at all.
    pub types: Option<Vec<TypeConstraint>>,
    pub fixed: Option<ElementDefinitionFixedTypeChoice>,
    pub pattern: Option<ElementDefinitionPatternTypeChoice>,
    pub binding: Option<Binding>,
    /// Children validated as plain fields, in snapshot order. Slices are not
    /// among them (they are validated through `slicings`); the elements that
    /// carry a slicing are, so that their cardinality, type, fixed or pattern
    /// value and children apply to every value, whichever slice it falls in.
    pub children: Vec<NodeId>,
    pub slicings: Vec<Slicing>,
    /// Every child in snapshot order, slicing roots and slices included, for
    /// finding the element a discriminator path names.
    pub lookup: Vec<NodeId>,
}

#[derive(Debug)]
pub(crate) struct Binding {
    pub required: bool,
    pub value_set: Option<String>,
}

#[derive(Debug)]
pub(crate) struct TypeConstraint {
    pub code: Option<String>,
    pub profiles: Vec<CanonicalSlot>,
}

/// `ElementDefinition.max` as written, read two ways: plain elements treat a
/// missing max as unbounded and an unparsable one as an error, slices treat
/// both as `1`.
#[derive(Debug)]
pub(crate) enum MaxCardinality {
    Unbounded,
    Fixed(u64),
    Missing,
    Invalid(String),
}

#[derive(Debug)]
pub(crate) struct Slicing {
    /// The element that carries `slicing`.
    pub root: NodeId,
    /// `Err` when the slicing cannot be evaluated; reported when a value
    /// reaches the sliced element's parent, as before.
    pub discriminators: Result<Vec<Discriminator>, SlicingError>,
    pub slices: Vec<Slice>,
}

#[derive(Debug)]
pub(crate) struct Slice {
    pub node: NodeId,
    /// Index of this slice's first matcher in the graph; one per discriminator.
    pub matchers: usize,
}

#[derive(Debug)]
pub(crate) struct Discriminator {
    pub kind: DiscriminatorKind,
    pub path: DiscriminatorPath,
    /// The path relative to the slicing root's parent (`extension.url`), as the
    /// diagnostics name it.
    pub search_path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiscriminatorKind {
    /// `value` and `pattern`, which FHIR R4 gives the same rule.
    Value,
    Exists,
    Type,
    Profile,
    Null,
    Unknown,
}

/// A discriminator path, compiled so the common forms need no `FHIRPath` engine.
#[derive(Debug)]
pub(crate) enum DiscriminatorPath {
    /// `$this`
    This,
    /// A dotted chain of member names such as `coding.code`.
    Members(Vec<String>),
    /// Anything else, evaluated by the `FHIRPath` engine.
    Expression(String),
}

#[derive(Debug)]
pub(crate) enum SlicingError {
    NoDiscriminators,
    MissingPath,
    UnsupportedPath(String),
}

#[derive(Debug)]
pub(crate) enum CompileError {
    /// The snapshot has no elements, so there is no root element to validate.
    NoRootElement,
}

/// Compiles the snapshot of `sd`.
pub(crate) fn compile(sd: &StructureDefinition) -> Result<CompiledProfile, CompileError> {
    let elements: &[ElementDefinition] = sd
        .snapshot
        .as_ref()
        .map_or(&[], |snapshot| snapshot.element.as_slice());
    if elements.is_empty() {
        return Err(CompileError::NoRootElement);
    }

    let children = child_lists(elements);
    let mut canonicals = Canonicals::default();
    let mut matcher_count = 0;
    let nodes = elements
        .iter()
        .zip(&children)
        .map(|(element, kids)| {
            let (plain, slicings) = split_children(elements, kids, &mut matcher_count);
            Node {
                element_id: element.id.clone().unwrap_or_else(|| "unknown".to_string()),
                field: field_name(path_of(element)),
                segment: path_of(element)
                    .rsplit('.')
                    .next()
                    .unwrap_or("")
                    .to_string(),
                min: element.min.as_ref().and_then(|min| min.value).unwrap_or(0),
                max: max_cardinality(element),
                types: element.type_.as_ref().map(|types| {
                    types
                        .iter()
                        .map(|type_| TypeConstraint {
                            code: type_.code.value.clone(),
                            profiles: type_
                                .profile
                                .iter()
                                .flatten()
                                .filter_map(|canonical| canonical.value.as_deref())
                                .map(|canonical| canonicals.slot(canonical))
                                .collect(),
                        })
                        .collect()
                }),
                fixed: element.fixed.clone(),
                pattern: element.pattern.clone(),
                binding: element.binding.as_ref().map(|binding| Binding {
                    required: binding.strength == BindingStrength::required(),
                    value_set: binding
                        .valueSet
                        .as_ref()
                        .and_then(|value_set| value_set.value.clone()),
                }),
                children: plain,
                slicings,
                lookup: kids.clone(),
            }
        })
        .collect();

    Ok(CompiledProfile {
        id: sd.id.clone().unwrap_or_else(|| "unknown".to_string()),
        nodes,
        canonicals: canonicals.0,
        matcher_count,
    })
}

#[derive(Default)]
struct Canonicals(Vec<String>);

impl Canonicals {
    fn slot(&mut self, canonical: &str) -> CanonicalSlot {
        self.0
            .iter()
            .position(|known| known == canonical)
            .unwrap_or_else(|| {
                self.0.push(canonical.to_string());
                self.0.len() - 1
            })
    }
}

fn path_of(element: &ElementDefinition) -> &str {
    element.path.value.as_deref().unwrap_or("")
}

/// The direct children of every element, in one pass: an element's parent is
/// the nearest preceding element with a shorter path.
fn child_lists(elements: &[ElementDefinition]) -> Vec<Vec<NodeId>> {
    let mut children = vec![Vec::new(); elements.len()];
    let mut open: Vec<(usize, NodeId)> = Vec::new();
    for (index, element) in elements.iter().enumerate() {
        let depth = path_of(element).matches('.').count();
        while open
            .last()
            .is_some_and(|(open_depth, _)| *open_depth >= depth)
        {
            open.pop();
        }
        if let Some(&(_, parent)) = open.last() {
            children[parent].push(index);
        }
        open.push((depth, index));
    }
    children
}

/// Separates a parent's children into plain fields and slicing groups: an
/// element with `slicing` starts a group that takes every directly following
/// sibling with a `sliceName`. The element with the slicing is also a plain
/// field, so its own constraints are checked against every value.
fn split_children(
    elements: &[ElementDefinition],
    kids: &[NodeId],
    matcher_count: &mut usize,
) -> (Vec<NodeId>, Vec<Slicing>) {
    let mut plain = Vec::new();
    let mut slicings = Vec::new();
    let mut i = 0;
    while i < kids.len() {
        let child = kids[i];
        i += 1;
        plain.push(child);
        let Some(slicing) = elements[child].slicing.as_ref() else {
            continue;
        };
        let discriminators = slicing
            .discriminator
            .as_ref()
            .ok_or(SlicingError::NoDiscriminators)
            .and_then(|discriminators| {
                discriminators
                    .iter()
                    .map(|discriminator| compile_discriminator(&elements[child], discriminator))
                    .collect::<Result<Vec<_>, _>>()
            });
        let mut slices = Vec::new();
        while i < kids.len() && has_slice_name(&elements[kids[i]]) {
            slices.push(Slice {
                node: kids[i],
                matchers: *matcher_count,
            });
            *matcher_count += discriminators.as_ref().map_or(0, Vec::len);
            i += 1;
        }
        slicings.push(Slicing {
            root: child,
            discriminators,
            slices,
        });
    }
    (plain, slicings)
}

fn has_slice_name(element: &ElementDefinition) -> bool {
    element
        .sliceName
        .as_ref()
        .is_some_and(|name| name.value.is_some())
}

fn max_cardinality(element: &ElementDefinition) -> MaxCardinality {
    match element.max.as_ref().and_then(|max| max.value.as_deref()) {
        None => MaxCardinality::Missing,
        Some("*") => MaxCardinality::Unbounded,
        Some(max) => max.parse().map_or_else(
            |_| MaxCardinality::Invalid(max.to_string()),
            MaxCardinality::Fixed,
        ),
    }
}

fn compile_discriminator(
    root: &ElementDefinition,
    discriminator: &ElementDefinitionSlicingDiscriminator,
) -> Result<Discriminator, SlicingError> {
    let path = discriminator
        .path
        .value
        .as_deref()
        .ok_or(SlicingError::MissingPath)?;
    if path.contains("ofType(") || path.contains("resolve()") || path.contains("extension(") {
        return Err(SlicingError::UnsupportedPath(path.to_string()));
    }
    let kind = &discriminator.type_;
    let kind = if kind == &DiscriminatorType::value() || kind == &DiscriminatorType::pattern() {
        DiscriminatorKind::Value
    } else if kind == &DiscriminatorType::exists() {
        DiscriminatorKind::Exists
    } else if kind == &DiscriminatorType::type_() {
        DiscriminatorKind::Type
    } else if kind == &DiscriminatorType::profile() {
        DiscriminatorKind::Profile
    } else if kind == &DiscriminatorType::null() {
        DiscriminatorKind::Null
    } else {
        DiscriminatorKind::Unknown
    };
    Ok(Discriminator {
        kind,
        path: compile_path(path),
        search_path: search_path(path_of(root), path),
    })
}

/// The discriminator path relative to the slicing root's parent, as the
/// diagnostics have always named it: the root path without its type prefix,
/// then the discriminator path with `$this` removed.
fn search_path(root_path: &str, discriminator_path: &str) -> String {
    let parent = root_path
        .find('.')
        .map_or("", |first_dot| &root_path[first_dot + 1..]);
    let relative = discriminator_path.replace("$this", "");
    if relative.is_empty() {
        parent.to_string()
    } else {
        format!("{parent}.{relative}")
    }
}

fn compile_path(path: &str) -> DiscriminatorPath {
    let path = path.trim();
    if path == "$this" {
        return DiscriminatorPath::This;
    }
    let members = path.strip_prefix("$this.").unwrap_or(path);
    let segments: Vec<&str> = members.split('.').collect();
    if segments.iter().all(|segment| is_identifier(segment)) {
        DiscriminatorPath::Members(segments.iter().map(ToString::to_string).collect())
    } else {
        DiscriminatorPath::Expression(path.to_string())
    }
}

fn is_identifier(segment: &str) -> bool {
    let mut chars = segment.chars();
    chars
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}
