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
    with_discriminator_values(path, value, |values| match kind {
        DiscriminatorKind::Exists => Ok(!values.is_empty()),
        DiscriminatorKind::Value => matches_value(values, target, binding),
        DiscriminatorKind::Type => matches_type(values, target),
        DiscriminatorKind::Profile => Err(OperationOutcomeError::error(
            IssueType::not_supported(),
            "Profile discriminator type is not supported".to_string(),
        )),
        DiscriminatorKind::Null => Err(OperationOutcomeError::error(
            IssueType::not_supported(),
            "Null discriminator type is not supported".to_string(),
        )),
        DiscriminatorKind::Unknown => Err(OperationOutcomeError::error(
            IssueType::not_supported(),
            "Unsupported discriminator type".to_string(),
        )),
    })
}

/// The values `path` selects on `value`, handed to `f` while they are alive.
///
/// `$this` and member chains are walked directly, the way the `FHIRPath` engine
/// walks a member (`get_field`, then `flatten`). Any other expression goes to
/// the engine, whose future completes at once for an expression that needs no
/// external service.
fn with_discriminator_values<T>(
    path: &DiscriminatorPath,
    value: &dyn MetaValue,
    f: impl FnOnce(&[&dyn MetaValue]) -> Result<T, OperationOutcomeError>,
) -> Result<T, OperationOutcomeError> {
    match path {
        DiscriminatorPath::This => f(&[value]),
        DiscriminatorPath::Members(segments) => {
            let mut current: Vec<&dyn MetaValue> = vec![value];
            let mut next = Vec::new();
            for segment in segments {
                next.clear();
                next.extend(
                    current
                        .iter()
                        .filter_map(|value| value.get_field(segment))
                        .flat_map(items),
                );
                std::mem::swap(&mut current, &mut next);
                if current.is_empty() {
                    break;
                }
            }
            f(&current)
        }
        DiscriminatorPath::Expression(expression) => {
            let evaluation = pin!(FP_ENGINE.evaluate(expression, vec![value]));
            match evaluation.poll(&mut Context::from_waker(Waker::noop())) {
                Poll::Ready(Ok(context)) => f(context.values()),
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

fn matches_type(values: &[&dyn MetaValue], target: &Node) -> Result<bool, OperationOutcomeError> {
    let expected = target.types.as_ref().ok_or_else(|| {
        OperationOutcomeError::error(
            IssueType::invalid(),
            "Slice value element definition must have types for type discriminator".to_string(),
        )
    })?;
    let found: HashSet<&str> = values.iter().map(|value| value.fhir_type()).collect();
    Ok(expected.iter().any(|type_| {
        type_
            .code
            .as_deref()
            .is_some_and(|code| found.contains(code))
    }))
}

fn matches_value(
    values: &[&dyn MetaValue],
    target: &Node,
    binding: &BindingCodes,
) -> Result<bool, OperationOutcomeError> {
    if let Some(fixed) = target.fixed.as_ref() {
        for value in values {
            if is_equal(*value, fixed)? {
                return Ok(true);
            }
        }
        return Ok(false);
    }
    if let Some(pattern) = target.pattern.as_ref() {
        for value in values {
            if validate_pattern(*value, pattern)? {
                return Ok(true);
            }
        }
        return Ok(false);
    }
    match binding {
        BindingCodes::Codes(codes) => Ok(values.iter().any(|value| codes.matches(*value))),
        BindingCodes::None | BindingCodes::Unresolvable(_) => Err(OperationOutcomeError::error(
            IssueType::invalid(),
            "Slice value element definition must have a fixed value, a pattern or a binding for value discriminator"
                .to_string(),
        )),
    }
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

/// `None` when the value set at `url`, or one it includes, cannot be resolved.
/// Every resource read goes into `sources`.
///
/// # Errors
///
/// Returns the resolver's error.
pub(crate) async fn value_set_codes(
    resolver: &impl CanonicalResolver,
    url: &str,
    depth: u8,
    sources: &mut Vec<Weak<Resource>>,
) -> Result<Option<ValueSetCodes>, OperationOutcomeError> {
    let Some(depth) = depth.checked_sub(1) else {
        return Ok(None);
    };
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
    let include = compose_rules(resolver, &compose.include, depth, sources).await?;
    let exclude = compose_rules(resolver, exclude, depth, sources).await?;
    Ok(include
        .zip(exclude)
        .map(|(include, exclude)| ValueSetCodes { include, exclude }))
}

async fn compose_rules(
    resolver: &impl CanonicalResolver,
    rules: &[ValueSetComposeInclude],
    depth: u8,
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
            let Some(value_set) = Box::pin(value_set_codes(resolver, url, depth, sources)).await?
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
