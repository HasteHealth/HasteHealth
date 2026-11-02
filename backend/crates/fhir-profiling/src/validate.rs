//! Walks a value and a compiled profile together and collects the issues.
//!
//! The walk is synchronous: everything that needed the resolver was done when
//! the [`ProfileGraph`] was assembled. Values are passed down as references, so
//! no path is parsed or re-walked from the root; the location of an issue is
//! rendered from the segments on the stack only when an issue is produced.

use std::fmt::Write as _;

use haste_fhir_model::r4::generated::{
    resources::{OperationOutcome, OperationOutcomeIssue},
    terminology::{BoundCode, IssueSeverity, IssueType},
    types::FHIRString,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_reflect::MetaValue;

use crate::{
    compile::{MaxCardinality, Node, NodeId, Slicing, SlicingError},
    graph::{BindingCodes, GraphProfile, Link, ProfileGraph, TargetError},
    slicing::{items, matches_discriminator},
    validators::{fixed_value::is_equal, pattern::validate_pattern},
};

/// One step of the path from the root value to the value being validated.
#[derive(Clone, Copy)]
enum Segment<'g> {
    Field(&'g str),
    Index(usize),
}

struct State<'g> {
    location: Vec<Segment<'g>>,
    issues: Vec<OperationOutcomeIssue>,
}

/// Validates `value` against the graph's root profile.
///
/// # Errors
///
/// Returns an error when the profile cannot be evaluated: a nested profile
/// that does not resolve, a discriminator that is not supported, an invalid
/// cardinality.
pub(crate) fn validate(
    graph: &ProfileGraph,
    value: &dyn MetaValue,
) -> Result<Vec<OperationOutcomeIssue>, OperationOutcomeError> {
    let mut state = State {
        location: Vec::with_capacity(16),
        issues: Vec::new(),
    };
    validate_node(graph, &mut state, 0, 0, Some(value))?;
    Ok(state.issues)
}

/// The location string of a value, in the JSON pointer form the issues use.
fn location(segments: &[Segment<'_>]) -> String {
    let mut rendered = String::new();
    for segment in segments {
        rendered.push('/');
        match segment {
            Segment::Field(field) => {
                if field.contains(['~', '/']) {
                    rendered.push_str(&field.replace('~', "~0").replace('/', "~1"));
                } else {
                    rendered.push_str(field);
                }
            }
            Segment::Index(index) => {
                let _ = write!(rendered, "{index}");
            }
        }
    }
    rendered
}

fn issue(
    location: String,
    severity: BoundCode<IssueSeverity>,
    code: BoundCode<IssueType>,
    diagnostic: String,
) -> OperationOutcomeIssue {
    OperationOutcomeIssue {
        severity,
        code,
        diagnostics: Some(Box::new(FHIRString {
            value: Some(diagnostic),
            ..Default::default()
        })),
        location: Some(vec![FHIRString {
            value: Some(location),
            ..Default::default()
        }]),
        ..Default::default()
    }
}

/// Validates the values a field holds (`None` when the field is absent)
/// against `node`: their cardinality, then each value.
fn validate_node<'g>(
    graph: &'g ProfileGraph,
    state: &mut State<'g>,
    profile: usize,
    node: NodeId,
    value: Option<&dyn MetaValue>,
) -> Result<(), OperationOutcomeError> {
    let definition = &graph.profiles[profile].compiled.nodes[node];
    let count = value.map_or(0, |value| items(value).count());
    check_cardinality(state, definition, count)?;
    let Some(value) = value else {
        return Ok(());
    };
    if value.is_many() {
        for (index, item) in items(value).enumerate() {
            state.location.push(Segment::Index(index));
            let result = validate_item(graph, state, profile, node, item);
            state.location.pop();
            result?;
        }
        Ok(())
    } else {
        validate_item(graph, state, profile, node, value)
    }
}

fn check_cardinality(
    state: &mut State<'_>,
    definition: &Node,
    count: usize,
) -> Result<(), OperationOutcomeError> {
    let count = count as u64;
    if count < definition.min {
        state.issues.push(issue(
            location(&state.location),
            IssueSeverity::error(),
            IssueType::required(),
            format!(
                "Element: '{}' Minimum number of required values not met expected at least '{}', found '{count}'",
                definition.element_id, definition.min
            ),
        ));
        return Ok(());
    }
    match &definition.max {
        MaxCardinality::Unbounded | MaxCardinality::Missing => Ok(()),
        MaxCardinality::Fixed(max) => {
            if count > *max {
                state.issues.push(issue(
                    location(&state.location),
                    IssueSeverity::error(),
                    IssueType::required(),
                    format!(
                        "Element: '{}' Too many values: expected at most '{max}', found '{count}'",
                        definition.element_id
                    ),
                ));
            }
            Ok(())
        }
        MaxCardinality::Invalid(max) => Err(OperationOutcomeError::error(
            IssueType::exception(),
            format!("Invalid max cardinality: {max}"),
        )),
    }
}

/// Validates one value against `node`: its slicings, its type and the
/// profiles that type names, its pattern and fixed value, then its children.
fn validate_item<'g>(
    graph: &'g ProfileGraph,
    state: &mut State<'g>,
    profile: usize,
    node: NodeId,
    item: &dyn MetaValue,
) -> Result<(), OperationOutcomeError> {
    let graph_profile = &graph.profiles[profile];
    let definition = &graph_profile.compiled.nodes[node];

    for slicing in &definition.slicings {
        validate_slicing(graph, state, profile, slicing, item)?;
    }

    validate_types(graph, state, profile, definition, item)?;

    if let Some(pattern) = definition.pattern.as_ref()
        && !validate_pattern(item, pattern)?
    {
        state.issues.push(issue(
            location(&state.location),
            IssueSeverity::error(),
            IssueType::value(),
            format!("Value does not match pattern: {pattern:?}"),
        ));
    }

    if let Some(fixed) = definition.fixed.as_ref()
        && !is_equal(item, fixed)?
    {
        state.issues.push(issue(
            location(&state.location),
            IssueSeverity::error(),
            IssueType::value(),
            format!("Value does not match fixed value: {fixed:?}"),
        ));
    }

    for &child in &definition.children {
        let child_definition = &graph_profile.compiled.nodes[child];
        state.location.push(Segment::Field(&child_definition.field));
        let result = validate_node(
            graph,
            state,
            profile,
            child,
            item.get_field(&child_definition.field),
        );
        state.location.pop();
        result?;
    }

    Ok(())
}

/// Whether a value of `value_type` may occupy an element typed `allowed`.
fn conformant_to_type(allowed: Option<&str>, value_type: &str) -> bool {
    match allowed {
        Some("Resource" | "DomainResource") => true,
        _ => allowed == Some(value_type),
    }
}

/// Checks the value's type against the element's, and validates the value
/// against every profile the matching type names.
fn validate_types<'g>(
    graph: &'g ProfileGraph,
    state: &mut State<'g>,
    profile: usize,
    definition: &Node,
    item: &dyn MetaValue,
) -> Result<(), OperationOutcomeError> {
    let value_type = item.fhir_type();
    // FHIRPath system types are not valid types to constrain to.
    if value_type.starts_with("http://hl7.org/fhirpath/System.") {
        return Ok(());
    }
    let Some(types) = definition.types.as_ref() else {
        return Ok(());
    };
    let Some(matching) = types
        .iter()
        .find(|type_| conformant_to_type(type_.code.as_deref(), value_type))
    else {
        // Reported without a location, as it always has been.
        state.issues.push(issue(
            String::new(),
            IssueSeverity::error(),
            IssueType::value(),
            format!(
                "Type '{value_type}' is not allowed for element '{}'",
                definition.element_id
            ),
        ));
        return Ok(());
    };
    for slot in &matching.profiles {
        match &graph.profiles[profile].links[*slot] {
            Link::Profile(nested) => validate_node(graph, state, *nested, 0, Some(item))?,
            link => return Err(link.error()),
        }
    }
    Ok(())
}

/// A value of a sliced field and its index in the field's list.
type Candidate<'v> = (Option<usize>, &'v dyn MetaValue);

fn validate_slicing<'g>(
    graph: &'g ProfileGraph,
    state: &mut State<'g>,
    profile: usize,
    slicing: &'g Slicing,
    item: &dyn MetaValue,
) -> Result<(), OperationOutcomeError> {
    let graph_profile = &graph.profiles[profile];
    let nodes = &graph_profile.compiled.nodes;
    let root = &nodes[slicing.root];

    let candidates: Vec<Candidate<'_>> = match item.get_field(&root.field) {
        None => Vec::new(),
        Some(values) if values.is_many() => items(values)
            .enumerate()
            .map(|(index, value)| (Some(index), value))
            .collect(),
        Some(value) => vec![(None, value)],
    };

    let discriminators = slicing.discriminators.as_ref().map_err(slicing_error)?;

    // Each value goes to the first slice it matches, in slice order.
    let mut remaining = candidates;
    let mut assigned: Vec<Vec<Candidate<'_>>> = Vec::with_capacity(slicing.slices.len());
    for slice in &slicing.slices {
        let mut in_slice = Vec::new();
        let mut rest = Vec::new();
        for candidate in remaining {
            state.location.push(Segment::Field(&root.field));
            if let Some(index) = candidate.0 {
                state.location.push(Segment::Index(index));
            }
            let is_match = (|| {
                for (offset, discriminator) in discriminators.iter().enumerate() {
                    let matcher = &graph_profile.matchers[slice.matchers + offset];
                    let (target_profile, target_node) = match &matcher.target {
                        Ok(target) => *target,
                        Err(TargetError::Link(link)) => return Err(link.error()),
                        Err(TargetError::NotFound) => {
                            return Err(OperationOutcomeError::error(
                                IssueType::invalid(),
                                format!(
                                    "'{}' Failed to find element definition for discriminator path '{}'",
                                    graph_profile.compiled.id, discriminator.search_path
                                ),
                            ));
                        }
                    };
                    if let BindingCodes::Unresolvable(url) = &matcher.binding {
                        return Err(unresolvable_value_set(
                            graph_profile,
                            &nodes[slice.node],
                            url,
                            &location(&state.location),
                        ));
                    }
                    let target = &graph.profiles[target_profile].compiled.nodes[target_node];
                    if !matches_discriminator(
                        discriminator.kind,
                        &discriminator.path,
                        target,
                        &matcher.binding,
                        candidate.1,
                    )? {
                        return Ok(false);
                    }
                }
                Ok(true)
            })();
            if candidate.0.is_some() {
                state.location.pop();
            }
            state.location.pop();
            if is_match? {
                in_slice.push(candidate);
            } else {
                rest.push(candidate);
            }
        }
        assigned.push(in_slice);
        remaining = rest;
    }

    for (slice, values) in slicing.slices.iter().zip(&assigned) {
        let slice_definition = &nodes[slice.node];
        validate_slice_cardinality(state, graph_profile, root, slice_definition, values);
        for (index, value) in values {
            state.location.push(Segment::Field(&root.field));
            if let Some(index) = index {
                state.location.push(Segment::Index(*index));
            }
            let result = validate_item(graph, state, profile, slice.node, *value);
            if index.is_some() {
                state.location.pop();
            }
            state.location.pop();
            result?;
        }
    }

    Ok(())
}

fn slicing_error(error: &SlicingError) -> OperationOutcomeError {
    match error {
        SlicingError::NoDiscriminators => OperationOutcomeError::error(
            IssueType::invalid(),
            "Invalid slicing discriminator configuration".to_string(),
        ),
        SlicingError::MissingPath => OperationOutcomeError::error(
            IssueType::exception(),
            "Discriminator path is missing".to_string(),
        ),
        SlicingError::UnsupportedPath(path) => OperationOutcomeError::error(
            IssueType::not_supported(),
            format!("Discriminator path '{path}' is not supported"),
        ),
    }
}

fn unresolvable_value_set(
    graph_profile: &GraphProfile,
    slice: &Node,
    url: &str,
    location: &str,
) -> OperationOutcomeError {
    OperationOutcomeError::unprocessable(OperationOutcome {
        issue: vec![issue(
            location.to_string(),
            IssueSeverity::error(),
            IssueType::not_found(),
            format!(
                "Profile: '{}' Element: '{}' The slice's ValueSet '{url}' could not be resolved",
                graph_profile.compiled.id, slice.element_id,
            ),
        )],
        ..Default::default()
    })
}

/// A slice's cardinality, read the way slices always were: a missing or
/// unparsable max is `1`.
fn slice_cardinality(slice: &Node) -> (u64, Option<u64>) {
    let max = match &slice.max {
        MaxCardinality::Unbounded => None,
        MaxCardinality::Fixed(max) => Some(*max),
        MaxCardinality::Missing | MaxCardinality::Invalid(_) => Some(1),
    };
    (slice.min, max)
}

fn validate_slice_cardinality<'g>(
    state: &mut State<'g>,
    graph_profile: &GraphProfile,
    root: &'g Node,
    slice: &Node,
    values: &[Candidate<'_>],
) {
    let (min, max) = slice_cardinality(slice);
    let count = values.len() as u64;
    if count >= min && max.is_none_or(|max| count <= max) {
        return;
    }
    // Issues point at the first value of the slice, or nowhere when it has none.
    let first = values.first().map_or_else(String::new, |(index, _)| {
        state.location.push(Segment::Field(&root.field));
        if let Some(index) = index {
            state.location.push(Segment::Index(*index));
        }
        let rendered = location(&state.location);
        if index.is_some() {
            state.location.pop();
        }
        state.location.pop();
        rendered
    });
    if count < min {
        state.issues.push(issue(
            first.clone(),
            IssueSeverity::error(),
            IssueType::value(),
            format!(
                "Profile: '{}' Element: '{}' Minimum number of required values not met expected at least '{min}', found '{count}'",
                graph_profile.compiled.id, slice.element_id
            ),
        ));
    }
    if let Some(max) = max
        && count > max
    {
        state.issues.push(issue(
            first,
            IssueSeverity::error(),
            IssueType::value(),
            format!("Cardinality too high: expected at most '{max}', found '{count}'"),
        ));
    }
}
