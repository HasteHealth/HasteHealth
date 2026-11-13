//! Decides which slice a value belongs to: evaluates a discriminator path on
//! the value and compares the result with what the slice's element definition
//! requires (`fixed[x]`, `pattern[x]`, a required binding, or the allowed
//! types).

use std::{
    collections::HashSet,
    future::Future,
    pin::pin,
    sync::{Arc, LazyLock, Weak},
    task::{Context, Poll, Waker},
};

use haste_fhir_client::canonical_resolver::CanonicalResolver;
use haste_fhir_model::r4::{
    conversion::downcast_string,
    generated::{
        resources::{Resource, ResourceType, ValueSetComposeInclude},
        terminology::IssueType,
        types::{CodeableConcept, Coding},
    },
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_reflect::MetaValue;

use crate::{
    compile::{DiscriminatorKind, DiscriminatorPath, Node},
    graph::BindingCodes,
    validators::{fixed_value::is_equal, pattern::validate_pattern},
};

static FP_ENGINE: LazyLock<haste_fhirpath::FPEngine> = LazyLock::new(haste_fhirpath::FPEngine::new);

/// Whether `value` belongs to the slice whose discriminating element is
/// `target`, judged by one discriminator.
pub(crate) fn matches_discriminator(
    kind: DiscriminatorKind,
    path: &DiscriminatorPath,
    target: &Node,
    binding: &BindingCodes,
    value: &dyn MetaValue,
) -> Result<bool, OperationOutcomeError> {
    match kind {
        DiscriminatorKind::Exists => any_discriminator_value(path, value, |_| Ok(true)),
        DiscriminatorKind::Value => {
            if let Some(fixed) = target.fixed.as_ref() {
                return any_discriminator_value(path, value, |value| is_equal(value, fixed));
            }
            if let Some(pattern) = target.pattern.as_ref() {
                return any_discriminator_value(path, value, |value| {
                    validate_pattern(value, pattern)
                });
            }
            match binding {
                BindingCodes::Codes(codes) => {
                    any_discriminator_value(path, value, |value| Ok(codes.matches(value)))
                }
                BindingCodes::None | BindingCodes::Unresolvable(_) => after_path(
                    path,
                    value,
                    OperationOutcomeError::error(
                        IssueType::invalid(),
                        "Slice value element definition must have a fixed value, a pattern or a binding for value discriminator"
                            .to_string(),
                    ),
                ),
            }
        }
        DiscriminatorKind::Type => {
            let Some(expected) = target.types.as_ref() else {
                return after_path(
                    path,
                    value,
                    OperationOutcomeError::error(
                        IssueType::invalid(),
                        "Slice value element definition must have types for type discriminator"
                            .to_string(),
                    ),
                );
            };
            any_discriminator_value(path, value, |value| {
                let found = value.fhir_type();
                Ok(expected
                    .iter()
                    .any(|type_| type_.code.as_deref() == Some(found)))
            })
        }
        DiscriminatorKind::Profile => after_path(
            path,
            value,
            OperationOutcomeError::error(
                IssueType::not_supported(),
                "Profile discriminator type is not supported".to_string(),
            ),
        ),
        DiscriminatorKind::Null => after_path(
            path,
            value,
            OperationOutcomeError::error(
                IssueType::not_supported(),
                "Null discriminator type is not supported".to_string(),
            ),
        ),
        DiscriminatorKind::Unknown => after_path(
            path,
            value,
            OperationOutcomeError::error(
                IssueType::not_supported(),
                "Unsupported discriminator type".to_string(),
            ),
        ),
    }
}

/// `error`, unless evaluating `path` on `value` fails first: a path the engine
/// cannot evaluate is reported ahead of what the discriminator lacks.
fn after_path(
    path: &DiscriminatorPath,
    value: &dyn MetaValue,
    error: OperationOutcomeError,
) -> Result<bool, OperationOutcomeError> {
    any_discriminator_value(path, value, |_| Ok(false))?;
    Err(error)
}

/// Whether any value `path` selects on `value` passes `test`, stopping at the
/// first that does (or at the first error).
///
/// `$this` and member chains are walked directly, the way the `FHIRPath` engine
/// walks a member (`get_field`, then `flatten`), visiting values in the order
/// the engine would return them. Any other expression goes to the engine, whose
/// future completes at once for an expression that needs no external service.
fn any_discriminator_value(
    path: &DiscriminatorPath,
    value: &dyn MetaValue,
    mut test: impl FnMut(&dyn MetaValue) -> Result<bool, OperationOutcomeError>,
) -> Result<bool, OperationOutcomeError> {
    match path {
        DiscriminatorPath::This => test(value),
        DiscriminatorPath::Members(segments) => any_member_value(value, segments, &mut test),
        DiscriminatorPath::Expression(expression) => {
            let evaluation = pin!(FP_ENGINE.evaluate(expression, vec![value]));
            match evaluation.poll(&mut Context::from_waker(Waker::noop())) {
                Poll::Ready(Ok(context)) => {
                    for value in context.values() {
                        if test(*value)? {
                            return Ok(true);
                        }
                    }
                    Ok(false)
                }
                Poll::Ready(Err(err)) => Err(OperationOutcomeError::error(
                    IssueType::exception(),
                    format!("Failed to evaluate FHIRPath expression for discriminator: {err}"),
                )),
                Poll::Pending => Err(OperationOutcomeError::error(
                    IssueType::not_supported(),
                    format!("Discriminator path '{expression}' is not supported"),
                )),
            }
        }
    }
}

/// Whether any value the member chain `segments` reaches from `value` passes
/// `test`.
fn any_member_value<F>(
    value: &dyn MetaValue,
    segments: &[String],
    test: &mut F,
) -> Result<bool, OperationOutcomeError>
where
    F: FnMut(&dyn MetaValue) -> Result<bool, OperationOutcomeError>,
{
    let Some((segment, rest)) = segments.split_first() else {
        return test(value);
    };
    let Some(field) = value.get_field(segment) else {
        return Ok(false);
    };
    for item in items(field) {
        if any_member_value(item, rest, test)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// The values a field holds: each element of a list, or the value itself.
pub(crate) fn items(value: &dyn MetaValue) -> impl Iterator<Item = &dyn MetaValue> {
    (0..).map_while(move |index| {
        if value.is_many() {
            value.get_index(index)
        } else {
            (index == 0).then_some(value)
        }
    })
}

/// The codes a value set's `compose` allows, read without expanding it. A
/// listed concept matches on its system and code, a whole system on the system
/// alone. Filters are not evaluated, so an include or exclude with one allows
/// no codes.
#[derive(Debug, Default)]
pub(crate) struct ValueSetCodes {
    include: Vec<ComposeRule>,
    exclude: Vec<ComposeRule>,
}

/// One `compose.include` or `compose.exclude`: a code is in it when it meets
/// every part given.
#[derive(Debug)]
struct ComposeRule {
    system: Option<String>,
    /// `None` for every code in `system`.
    codes: Option<HashSet<String>>,
    value_sets: Vec<ValueSetCodes>,
}

impl ComposeRule {
    /// `system` is `None` for a bare `code`, whose system the binding implies.
    fn contains(&self, system: Option<&str>, code: &str) -> bool {
        (system.is_none() || self.system.is_none() || system == self.system.as_deref())
            && self.codes.as_ref().is_none_or(|codes| codes.contains(code))
            && self
                .value_sets
                .iter()
                .all(|value_set| value_set.contains(system, code))
    }
}

impl ValueSetCodes {
    fn contains(&self, system: Option<&str>, code: &str) -> bool {
        self.include.iter().any(|rule| rule.contains(system, code))
            && !self.exclude.iter().any(|rule| rule.contains(system, code))
    }

    /// A `Coding` matches on its system and code, a `CodeableConcept` when any
    /// of its codings does, and a `code` or `string` on the code.
    fn matches(&self, value: &dyn MetaValue) -> bool {
        let coding_matches = |coding: &Coding| {
            let system = coding
                .system
                .as_ref()
                .and_then(|system| system.value.as_deref());
            let code = coding.code.as_ref().and_then(|code| code.value.as_deref());
            system
                .zip(code)
                .is_some_and(|(system, code)| self.contains(Some(system), code))
        };
        match value.fhir_type() {
            "Coding" => downcast::<Coding>(value).is_some_and(coding_matches),
            "CodeableConcept" => downcast::<CodeableConcept>(value)
                .and_then(|concept| concept.coding.as_ref())
                .is_some_and(|codings| codings.iter().any(coding_matches)),
            _ => downcast_string(value).is_ok_and(|code| self.contains(None, &code)),
        }
    }
}

/// `value` as a `T`, boxed or not.
fn downcast<T: 'static>(value: &dyn MetaValue) -> Option<&T> {
    value.as_any().downcast_ref()
}

/// How deeply value sets may include each other.
const MAX_VALUE_SET_DEPTH: usize = 8;

/// How many value sets one expansion may read in all; a broad chain of
/// includes multiplies at every level.
const MAX_VALUE_SETS_READ: usize = 512;

/// One expansion of a value set's includes: the value sets it may still read
/// and the chain of includes it is following.
pub(crate) struct ValueSetWalk {
    remaining: usize,
    chain: Vec<String>,
}

impl Default for ValueSetWalk {
    fn default() -> Self {
        Self {
            remaining: MAX_VALUE_SETS_READ,
            chain: Vec::new(),
        }
    }
}

/// `None` when the value set at `url`, or one it includes, cannot be resolved.
/// Every resource read goes into `sources`.
///
/// # Errors
///
/// Returns the resolver's error.
pub(crate) async fn value_set_codes(
    resolver: &impl CanonicalResolver,
    url: &str,
    walk: &mut ValueSetWalk,
    sources: &mut Vec<Weak<Resource>>,
) -> Result<Option<ValueSetCodes>, OperationOutcomeError> {
    // A value set that includes itself, directly or through others, adds
    // nothing the outer expansion does not already hold.
    if walk.chain.iter().any(|seen| seen == url) {
        return Ok(Some(ValueSetCodes::default()));
    }
    if walk.chain.len() >= MAX_VALUE_SET_DEPTH {
        return Ok(None);
    }
    let Some(remaining) = walk.remaining.checked_sub(1) else {
        return Ok(None);
    };
    walk.remaining = remaining;
    let resource = resolver.resolve(ResourceType::ValueSet, url).await?;
    let Some(resource) = resource else {
        return Ok(None);
    };
    sources.push(Arc::downgrade(&resource));
    let Resource::ValueSet(value_set) = resource.as_ref() else {
        return Ok(None);
    };
    let Some(compose) = &value_set.compose else {
        return Ok(Some(ValueSetCodes::default()));
    };
    let exclude = compose.exclude.as_deref().unwrap_or_default();
    walk.chain.push(url.to_string());
    let include = compose_rules(resolver, &compose.include, walk, sources).await;
    let exclude = compose_rules(resolver, exclude, walk, sources).await;
    walk.chain.pop();
    let (include, exclude) = (include?, exclude?);
    Ok(include
        .zip(exclude)
        .map(|(include, exclude)| ValueSetCodes { include, exclude }))
}

async fn compose_rules(
    resolver: &impl CanonicalResolver,
    rules: &[ValueSetComposeInclude],
    walk: &mut ValueSetWalk,
    sources: &mut Vec<Weak<Resource>>,
) -> Result<Option<Vec<ComposeRule>>, OperationOutcomeError> {
    let mut compose_rules = Vec::with_capacity(rules.len());
    for rule in rules {
        let mut value_sets = Vec::new();
        for url in rule
            .valueSet
            .iter()
            .flatten()
            .filter_map(|url| url.value.as_deref())
        {
            let Some(value_set) = Box::pin(value_set_codes(resolver, url, walk, sources)).await?
            else {
                return Ok(None);
            };
            value_sets.push(value_set);
        }
        let system = rule.system.as_ref().and_then(|system| system.value.clone());
        let codes = if rule.filter.is_some() || (system.is_none() && value_sets.is_empty()) {
            Some(HashSet::new())
        } else {
            rule.concept.as_ref().map(|concepts| {
                concepts
                    .iter()
                    .filter_map(|concept| concept.code.value.clone())
                    .collect()
            })
        };
        compose_rules.push(ComposeRule {
            system,
            codes,
            value_sets,
        });
    }
    Ok(Some(compose_rules))
}
