use crate::fhir_client::{
    ClientState, ServerCTX,
    middleware::{Backend, Middleware, Next, Output},
};
use haste_artifacts::ARTIFACT_RESOURCES;
use haste_fhir_client::request::{FHIRRequest, UpdateRequest};
use haste_fhir_model::r4::generated::{
    resources::{
        DeidentificationPolicy, DeidentificationPolicyRule, DeidentificationPolicyRuleParam,
        Resource, StructureDefinition,
    },
    terminology::{IssueType, StructureDefinitionKind},
};
use haste_fhir_operation_error::OperationOutcomeError;
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, LazyLock},
};

fn invalid(path: &str, message: &str) -> OperationOutcomeError {
    OperationOutcomeError::fatal(IssueType::invalid(), format!("{path}: {message}."))
}

/// What a rule matches. Variant order is the resolution precedence, so
/// sorting by it puts the most specific targets first.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TargetKey {
    Path(String),
    Extension(String),
    Type(String),
}

fn target_key(rule: &DeidentificationPolicyRule) -> Option<TargetKey> {
    let target = &rule.target;
    let path = target.path.as_ref().and_then(|v| v.value.clone());
    let kind = target.type_.as_ref().and_then(|v| v.value.clone());
    let extension = target.extension.as_ref().and_then(|v| v.value.clone());
    match (path, kind, extension) {
        (Some(path), None, None) => Some(TargetKey::Path(path)),
        (None, Some(kind), None) => Some(TargetKey::Type(kind)),
        (None, None, Some(url)) => Some(TargetKey::Extension(url)),
        _ => None,
    }
}

/// Element paths of every resource and datatype, so a target can be checked
/// without a lookup per save.
struct ElementIndex {
    /// Path to the type codes of the element and its `contentReference`.
    elements: HashMap<String, (Vec<String>, Option<String>)>,
    /// Non-abstract primitive and complex datatypes.
    datatypes: HashSet<String>,
    roots: HashSet<String>,
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

fn index_definition(index: &mut ElementIndex, sd: &StructureDefinition) {
    let name = sd.type_.value.clone().unwrap_or_default();
    if sd.kind == StructureDefinitionKind::primitive_type()
        || sd.kind == StructureDefinitionKind::complex_type()
    {
        if sd.abstract_.value != Some(true) {
            index.datatypes.insert(name.clone());
        }
    } else if sd.kind != StructureDefinitionKind::resource() {
        return;
    }
    index.roots.insert(name);

    for element in sd.snapshot.iter().flat_map(|s| &s.element) {
        let Some(path) = element.path.value.clone() else {
            continue;
        };
        let codes: Vec<String> = element
            .type_
            .iter()
            .flatten()
            .filter_map(|t| t.code.value.clone())
            .collect();
        let reference = element
            .contentReference
            .as_ref()
            .and_then(|r| r.value.as_ref())
            .map(|r| r.trim_start_matches('#').to_string());
        if let Some(stem) = path.strip_suffix("[x]") {
            for code in &codes {
                index.elements.insert(
                    format!("{stem}{}", capitalize(code)),
                    (vec![code.clone()], None),
                );
            }
        }
        index.elements.insert(path, (codes, reference));
    }
}

static ELEMENT_INDEX: LazyLock<ElementIndex> = LazyLock::new(|| {
    let mut index = ElementIndex {
        elements: HashMap::new(),
        datatypes: HashSet::new(),
        roots: HashSet::new(),
    };
    for resource in ARTIFACT_RESOURCES.iter() {
        if let Resource::StructureDefinition(sd) = resource {
            index_definition(&mut index, sd);
        }
    }
    index
});

/// Walks `Type.a.b`, stepping into a datatype or `contentReference` when the
/// element has no children of its own in the snapshot.
fn path_exists(index: &ElementIndex, path: &str) -> bool {
    let mut segments = path.split('.');
    let Some(root) = segments.next().filter(|r| index.roots.contains(*r)) else {
        return false;
    };
    let mut current = root.to_string();
    for segment in segments {
        let next = format!("{current}.{segment}");
        if index.elements.contains_key(&next) {
            current = next;
            continue;
        }
        let Some((codes, reference)) = index.elements.get(&current) else {
            return false;
        };
        let base = match (reference, codes.as_slice()) {
            (Some(reference), _) => reference.clone(),
            (None, [code]) if index.roots.contains(code) => code.clone(),
            _ => return false,
        };
        let next = format!("{base}.{segment}");
        if !index.elements.contains_key(&next) {
            return false;
        }
        current = next;
    }
    current.contains('.')
}

fn validate_target(index: &ElementIndex, key: &TargetKey, at: &str) -> Result<(), OperationOutcomeError> {
    match key {
        TargetKey::Path(path) if !path_exists(index, path) => {
            Err(invalid(at, &format!("unknown element path '{path}'")))
        }
        TargetKey::Type(kind) if !index.datatypes.contains(kind) => {
            Err(invalid(at, &format!("unknown datatype '{kind}'")))
        }
        _ => Ok(()),
    }
}

fn is_semver(version: &str) -> bool {
    let core_end = version.find(['-', '+']).unwrap_or(version.len());
    let (core, rest) = version.split_at(core_end);
    let numeric = |p: &str| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()) && (p == "0" || !p.starts_with('0'));
    let parts: Vec<&str> = core.split('.').collect();
    let tail_ok = rest.is_empty()
        || rest[1..].split(['-', '+', '.']).all(|s| {
            !s.is_empty() && s.bytes().all(|b| b.is_ascii_alphanumeric())
        });
    parts.len() == 3 && parts.iter().all(|p| numeric(p)) && tail_ok
}

const GENERALIZE_TARGETS: [&str; 3] = ["year", "month", "zip3"];

/// Rejects a param that is missing, or not allowed, for the action.
fn validate_param(
    action: &str,
    param: Option<&DeidentificationPolicyRuleParam>,
    at: &str,
) -> Result<(), OperationOutcomeError> {
    let empty = DeidentificationPolicyRuleParam::default();
    let p = param.unwrap_or(&empty);
    let present = [
        ("keep", p.keep.as_ref().is_some_and(|k| !k.is_empty())),
        ("to", p.to.is_some()),
        ("ageOver", p.ageOver.is_some()),
        ("rangeDays", p.rangeDays.is_some()),
        ("value", p.value.is_some()),
        ("percent", p.percent.is_some()),
    ];
    let (allowed, required): (&[&str], &[&str]) = match action {
        "cryptoHash" | "pseudonymize" => (&["keep"], &[]),
        "substitute" => (&["value"], &["value"]),
        "dateShift" => (&["rangeDays"], &["rangeDays"]),
        "generalize" => (&["to", "ageOver"], &["to"]),
        "perturb" => (&["percent"], &["percent"]),
        _ => (&[], &[]),
    };
    for (name, is_set) in present {
        if is_set && !allowed.contains(&name) {
            return Err(invalid(at, &format!("param '{name}' does not apply to action '{action}'")));
        }
        if !is_set && required.contains(&name) {
            return Err(invalid(at, &format!("action '{action}' requires param '{name}'")));
        }
    }
    if let Some(to) = p.to.as_ref().and_then(|t| t.value.as_deref())
        && !GENERALIZE_TARGETS.contains(&to)
    {
        return Err(invalid(at, &format!("param 'to' must be one of {GENERALIZE_TARGETS:?}")));
    }
    if let Some(percent) = p.percent.as_ref().and_then(|v| v.value)
        && !(percent > 0.0 && percent <= 100.0)
    {
        return Err(invalid(at, "param 'percent' must be in (0, 100]"));
    }
    Ok(())
}

fn validate_with(index: &ElementIndex, policy: &DeidentificationPolicy) -> Result<(), OperationOutcomeError> {
    let version = policy.version.value.as_deref().unwrap_or_default();
    if !is_semver(version) {
        return Err(invalid("DeidentificationPolicy.version", "must be a SemVer version such as 1.0.0"));
    }

    let mut seen = HashSet::new();
    for (i, rule) in policy.rule.iter().flatten().enumerate() {
        let at = format!("DeidentificationPolicy.rule[{i}]");
        let key = target_key(rule)
            .ok_or_else(|| invalid(&at, "target requires exactly one of path, type and extension"))?;
        validate_target(index, &key, &at)?;
        validate_param(rule.action.as_str().unwrap_or_default(), rule.param.as_ref(), &at)?;
        if !seen.insert(key) {
            return Err(invalid(&at, "duplicate target"));
        }
    }
    Ok(())
}

/// Orders rules by precedence then target so equal policies serialize equally
/// and diffs are semantic. Rules already validated to carry a target.
pub fn sort_rules(policy: &mut DeidentificationPolicy) {
    if let Some(rules) = policy.rule.as_mut() {
        rules.sort_by_cached_key(target_key);
    }
}

/// Resolves `child` against its `parent`: the child's rules override the
/// parent's by target identity, parent rules it does not mention carry over
/// and the child's `default` applies.
#[must_use]
pub fn merge(parent: &DeidentificationPolicy, child: &DeidentificationPolicy) -> DeidentificationPolicy {
    let mut by_target: HashMap<Option<TargetKey>, &DeidentificationPolicyRule> = HashMap::new();
    for rule in parent.rule.iter().chain(&child.rule).flatten() {
        by_target.insert(target_key(rule), rule);
    }
    let mut merged = child.clone();
    merged.rule = Some(by_target.into_values().cloned().collect());
    sort_rules(&mut merged);
    merged
}

/// Validates a `DeidentificationPolicy` on save and sorts its rules.
/// Resolving `basedOn` is left to the consumer of the policy.
pub struct DeidentificationPolicyValidation;

impl<B: Backend> Middleware<B> for DeidentificationPolicyValidation {
    fn call<'a>(
        &'a self,
        state: &'a ClientState<B>,
        ctx: Arc<ServerCTX<B>>,
        mut request: FHIRRequest,
        next: Next<'a, B>,
    ) -> Output<'a> {
        let resource = match &mut request {
            FHIRRequest::Create(create) => Some(&mut create.resource),
            FHIRRequest::Update(UpdateRequest::Instance(update)) => Some(&mut update.resource),
            FHIRRequest::Update(UpdateRequest::Conditional(update)) => Some(&mut update.resource),
            _ => None,
        };
        if let Some(Resource::DeidentificationPolicy(policy)) = resource {
            if let Err(error) = validate_with(&ELEMENT_INDEX, policy) {
                return Box::pin(std::future::ready(Err(error)));
            }
            sort_rules(policy);
        }
        next.run(state, ctx, request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy(json: &str) -> DeidentificationPolicy {
        serde_json::from_str(json).unwrap()
    }

    fn base(rules: &str) -> DeidentificationPolicy {
        policy(&format!(
            r#"{{"resourceType":"DeidentificationPolicy","url":"http://x/p","version":"1.0.0",
            "status":"active","default":"redact","rule":[{rules}]}}"#
        ))
    }

    fn check(rules: &str) -> Result<(), OperationOutcomeError> {
        validate_with(&ELEMENT_INDEX, &base(rules))
    }

    fn action_of<'a>(p: &'a DeidentificationPolicy, key: &TargetKey) -> &'a str {
        let rule = p.rule.iter().flatten().find(|r| target_key(r).as_ref() == Some(key)).unwrap();
        rule.action.as_str().unwrap()
    }

    #[test]
    fn accepts_valid_targets() {
        check(
            r#"{"target":{"path":"Patient.birthDate"},"action":"generalize","param":{"to":"year","ageOver":89}},
               {"target":{"path":"Patient.name.family"},"action":"redact"},
               {"target":{"path":"Observation.valueQuantity"},"action":"keep"},
               {"target":{"path":"Questionnaire.item.item.text"},"action":"keep"},
               {"target":{"path":"Address.postalCode"},"action":"generalize","param":{"to":"zip3"}},
               {"target":{"type":"dateTime"},"action":"dateShift","param":{"rangeDays":365}},
               {"target":{"type":"Identifier"},"action":"cryptoHash","param":{"keep":["system"]}},
               {"target":{"extension":"http://x/ext"},"action":"keep"}"#,
        )
        .unwrap();
    }

    #[test]
    fn rejects_unknown_targets() {
        let error = check(r#"{"target":{"path":"Patient.birthDay"},"action":"redact"}"#).unwrap_err();
        assert!(format!("{error:?}").contains("Patient.birthDay"));
        assert!(check(r#"{"target":{"path":"Nope.id"},"action":"redact"}"#).is_err());
        assert!(check(r#"{"target":{"path":"Patient"},"action":"redact"}"#).is_err());
        assert!(check(r#"{"target":{"type":"Patient"},"action":"redact"}"#).is_err());
        assert!(check(r#"{"target":{"type":"Element"},"action":"redact"}"#).is_err());
    }

    #[test]
    fn requires_exactly_one_target_kind() {
        assert!(check(r#"{"target":{},"action":"redact"}"#).is_err());
        assert!(check(r#"{"target":{"path":"Patient.id","type":"id"},"action":"redact"}"#).is_err());
    }

    #[test]
    fn rejects_params_that_do_not_fit_the_action() {
        let t = r#""target":{"type":"dateTime"}"#;
        assert!(check(&format!(r#"{{{t},"action":"redact","param":{{"rangeDays":1}}}}"#)).is_err());
        assert!(check(&format!(r#"{{{t},"action":"dateShift"}}"#)).is_err());
        assert!(check(&format!(r#"{{{t},"action":"dateShift","param":{{"value":"x"}}}}"#)).is_err());
        assert!(check(&format!(r#"{{{t},"action":"generalize","param":{{"to":"decade"}}}}"#)).is_err());
        assert!(check(&format!(r#"{{{t},"action":"substitute"}}"#)).is_err());
        assert!(check(&format!(r#"{{{t},"action":"perturb","param":{{"percent":0}}}}"#)).is_err());
    }

    #[test]
    fn rejects_duplicate_targets_and_bad_versions() {
        let rule = r#"{"target":{"type":"id"},"action":"redact"}"#;
        assert!(check(&format!("{rule},{rule}")).is_err());
        for bad in ["1.0", "v1.0.0", "01.0.0", "1.0.0-", ""] {
            let mut p = base(rule);
            p.version = Box::new(haste_fhir_model::r4::generated::types::FHIRString { value: Some(bad.into()), ..Default::default() });
            assert!(validate_with(&ELEMENT_INDEX, &p).is_err(), "{bad}");
        }
        assert!(is_semver("1.2.3-rc.1+build.5"));
    }

    #[test]
    fn sorts_by_precedence() {
        let mut p = base(
            r#"{"target":{"type":"id"},"action":"redact"},
               {"target":{"extension":"http://x/e"},"action":"keep"},
               {"target":{"path":"Patient.id"},"action":"keep"}"#,
        );
        sort_rules(&mut p);
        let keys: Vec<_> = p.rule.iter().flatten().filter_map(target_key).collect();
        assert!(matches!(keys[..], [TargetKey::Path(_), TargetKey::Extension(_), TargetKey::Type(_)]));
    }

    #[test]
    fn merge_overrides_adds_and_inherits() {
        let parent = base(
            r#"{"target":{"type":"HumanName"},"action":"redact"},
               {"target":{"path":"Patient.birthDate"},"action":"keep"}"#,
        );
        let mut child = base(
            r#"{"target":{"path":"Patient.birthDate"},"action":"generalize","param":{"to":"year"}},
               {"target":{"type":"id"},"action":"keep"}"#,
        );
        child.default = parent.default.clone();
        let merged = merge(&parent, &child);
        let rules = merged.rule.as_ref().unwrap();
        assert_eq!(rules.len(), 3);
        assert_eq!(action_of(&merged, &TargetKey::Path("Patient.birthDate".into())), "generalize");
        assert_eq!(action_of(&merged, &TargetKey::Type("HumanName".into())), "redact");
        assert_eq!(action_of(&merged, &TargetKey::Type("id".into())), "keep");
        assert_eq!(merged.default, child.default);
    }
}
