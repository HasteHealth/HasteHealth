//! What an `AccessPolicyV3`'s expressions reference, checked against the context schema
//! on save, and the manifest built from it: per rule, the attributes it needs and their
//! scopes. Deciding what to load from the policy text, once, keeps evaluation free of
//! discovery.

use haste_fhir_model::r4::generated::{
    resources::{
        AccessPolicyV3, AccessPolicyV3Manifest, AccessPolicyV3ManifestAttribute, AccessPolicyV3Rule,
    },
    terminology::{Accesspolicyv3attributescope, IssueType},
    types::{FHIRId, FHIRString},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhirpath::{ExternalConstant, external_constants, fhir_environment_variable};
use std::collections::HashMap;

/// The fields of `%request` (`RequestReflection` in haste-access-control).
const REQUEST_FIELDS: &[&str] = &[
    "channel",
    "id",
    "level",
    "parameters",
    "resource",
    "resource_type",
    "type",
];

/// The fields every `%principal` has; `%principal.<attributeId>` names a principal-scope
/// attribute instead.
const PRINCIPAL_FIELDS: &[&str] = &["practitioner", "user"];

/// `%target`, the instance an instance-level request reads or writes: a request-scope
/// attribute every policy has without declaring it.
const TARGET: &str = "target";

/// Request-scope attributes are fetched per request, so a policy gets few of them.
const REQUEST_ATTRIBUTE_LIMIT: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Scope {
    Principal,
    Request,
}

/// What a `%name` (and member) in a policy stands for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Resolved<'p> {
    Request,
    Principal,
    /// A FHIR environment variable such as `%sct`.
    Environment,
    Attribute(&'p str, Scope),
}

/// Where a reference occurs, which decides what it may read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Site {
    Target,
    Condition,
    Data,
    /// The parameters of an attribute's operation, with that attribute's scope.
    Operation(Scope),
}

/// The declared attributes by id.
type Declared<'p> = HashMap<&'p str, Scope>;

/// An attribute a rule references: its id and scope.
type Attribute<'p> = (&'p str, Scope);

fn invalid(path: &str, message: &str) -> OperationOutcomeError {
    OperationOutcomeError::fatal(IssueType::invalid(), format!("{path}: {message}."))
}

/// Checks every expression and substitution of `policy` against the context schema and
/// returns the manifest: one entry per rule that references attributes.
///
/// # Errors
///
/// Names the rule and the expression of the first unknown attribute or request field,
/// request-scope reference in a target or constraint, attribute depending on another
/// attribute, invalid `FHIRPath` or operation, or of too many request-scope attributes.
pub fn manifest(
    policy: &AccessPolicyV3,
) -> Result<Vec<AccessPolicyV3Manifest>, OperationOutcomeError> {
    let declared = declared_attributes(policy)?;
    let mut manifest = Vec::new();
    walk_rules(
        policy.rule.as_deref().unwrap_or_default(),
        "rule",
        &declared,
        &mut manifest,
    )?;

    let implicit_target = !declared.contains_key(TARGET)
        && manifest
            .iter()
            .flat_map(|entry| &entry.attribute)
            .any(|attribute| attribute.attributeId.value.as_deref() == Some(TARGET));
    let request_scoped = declared
        .values()
        .filter(|scope| **scope == Scope::Request)
        .count()
        + usize::from(implicit_target);
    if request_scoped > REQUEST_ATTRIBUTE_LIMIT {
        return Err(invalid(
            "AccessPolicyV3.attribute",
            &format!(
                "{request_scoped} request-scope attributes; the limit is {REQUEST_ATTRIBUTE_LIMIT}"
            ),
        ));
    }
    Ok(manifest)
}

/// The declared attributes by id, each with an operation of a supported type whose
/// parameters depend on built-ins only.
fn declared_attributes(policy: &AccessPolicyV3) -> Result<Declared<'_>, OperationOutcomeError> {
    let attributes = policy.attribute.as_deref().unwrap_or_default();
    let declared: Declared<'_> = attributes
        .iter()
        .enumerate()
        .map(|(index, attribute)| {
            let id = attribute.attributeId.value.as_deref().unwrap_or_default();
            let scope = match attribute.scope.as_str() {
                Some("principal") => Scope::Principal,
                Some("request") => Scope::Request,
                _ => {
                    return Err(invalid(
                        &format!("AccessPolicyV3.attribute[{index}].scope"),
                        &format!("attribute '{id}' has an unknown scope"),
                    ));
                }
            };
            Ok((id, scope))
        })
        .collect::<Result<_, _>>()?;

    for (index, attribute) in attributes.iter().enumerate() {
        let path = format!("AccessPolicyV3.attribute[{index}].operation");
        let id = attribute.attributeId.value.as_deref().unwrap_or_default();
        // The operation type is bound to read, search-type and search-system by its ValueSet.
        let Some(operation) = attribute.operation.as_ref() else {
            return Err(invalid(
                &path,
                &format!("attribute '{id}' needs an operation"),
            ));
        };
        if let Some(parameters) = operation
            .parameters
            .as_ref()
            .and_then(|p| p.value.as_deref())
        {
            let site = Site::Operation(declared[id]);
            for reference in substitutions(parameters) {
                check(&reference, site, &declared, parameters)
                    .map_err(|message| invalid(&format!("{path}.parameters"), &message))?;
            }
        }
    }
    Ok(declared)
}

/// Appends a manifest entry for every rule under `path` that references attributes.
fn walk_rules<'p>(
    rules: &'p [AccessPolicyV3Rule],
    path: &str,
    declared: &Declared<'p>,
    manifest: &mut Vec<AccessPolicyV3Manifest>,
) -> Result<(), OperationOutcomeError> {
    for (index, rule) in rules.iter().enumerate() {
        let path = format!("{path}[{index}]");
        let mut attributes: Vec<Attribute<'p>> = Vec::new();

        let gates = [
            (Site::Target, rule.target.as_ref().map(|t| &t.expression)),
            (
                Site::Condition,
                rule.condition.as_ref().map(|c| &c.expression),
            ),
        ];
        for (site, expression) in gates {
            let Some(expression) = expression.and_then(|e| e.value.as_deref()) else {
                continue;
            };
            let element = format!("AccessPolicyV3.{path}.{}", site_name(site));
            let references = external_constants(expression).map_err(|error| {
                invalid(
                    &element,
                    &format!("invalid FHIRPath in '{expression}': {error}"),
                )
            })?;
            collect(
                &references,
                site,
                declared,
                expression,
                &element,
                &mut attributes,
            )?;
        }
        for (data_index, data) in rule.data.iter().flatten().enumerate() {
            let search = data.search.value.as_deref().unwrap_or_default();
            let element = format!("AccessPolicyV3.{path}.data[{data_index}].search");
            let references: Vec<_> = substitutions(search).collect();
            collect(
                &references,
                Site::Data,
                declared,
                search,
                &element,
                &mut attributes,
            )?;
        }

        if !attributes.is_empty() {
            manifest.push(manifest_entry(&path, &attributes));
        }
        if let Some(nested) = rule.rule.as_deref() {
            walk_rules(nested, &format!("{path}.rule"), declared, manifest)?;
        }
    }
    Ok(())
}

/// Checks each reference at `site` and records the attributes among them, once each.
fn collect<'p>(
    references: &[ExternalConstant],
    site: Site,
    declared: &Declared<'p>,
    text: &str,
    element: &str,
    attributes: &mut Vec<Attribute<'p>>,
) -> Result<(), OperationOutcomeError> {
    for reference in references {
        let resolved =
            check(reference, site, declared, text).map_err(|message| invalid(element, &message))?;
        if let Resolved::Attribute(id, scope) = resolved
            && !attributes.iter().any(|(known, _)| *known == id)
        {
            attributes.push((id, scope));
        }
    }
    Ok(())
}

/// Resolves `reference` against the schema and checks that `site` may read it.
fn check<'p>(
    reference: &ExternalConstant,
    site: Site,
    declared: &Declared<'p>,
    text: &str,
) -> Result<Resolved<'p>, String> {
    let resolved =
        resolve(reference, declared).map_err(|message| format!("{message} in '{text}'"))?;
    allowed(site, resolved).map_err(|message| format!("{message} in '{text}'"))?;
    Ok(resolved)
}

/// What `%name.member` stands for, or why it is unknown.
fn resolve<'p>(
    reference: &ExternalConstant,
    declared: &Declared<'p>,
) -> Result<Resolved<'p>, String> {
    let member = reference.member.as_deref();
    match reference.name.as_str() {
        "request" => resolve_request(member),
        "principal" => resolve_principal(member, declared),
        name => resolve_variable(name, declared),
    }
}

/// `%request`, or one of its fields.
fn resolve_request<'p>(field: Option<&str>) -> Result<Resolved<'p>, String> {
    match field {
        Some(field) if !REQUEST_FIELDS.contains(&field) => {
            Err(format!("%request has no field '{field}'"))
        }
        _ => Ok(Resolved::Request),
    }
}

/// `%principal`, one of its built-in fields, or a principal-scope attribute read
/// through it.
fn resolve_principal<'p>(
    field: Option<&str>,
    declared: &Declared<'p>,
) -> Result<Resolved<'p>, String> {
    let Some(field) = field else {
        return Ok(Resolved::Principal);
    };
    if PRINCIPAL_FIELDS.contains(&field) {
        return Ok(Resolved::Principal);
    }
    match declared.get_key_value(field) {
        Some((id, Scope::Principal)) => Ok(Resolved::Attribute(id, Scope::Principal)),
        Some(_) => Err(format!(
            "'{field}' is a request-scope attribute, not a %principal attribute"
        )),
        None => Err(format!("unknown attribute '{field}'")),
    }
}

/// Any other `%name`: a declared attribute, the implicit `%target`, or a FHIR
/// environment variable such as `%sct`.
fn resolve_variable<'p>(name: &str, declared: &Declared<'p>) -> Result<Resolved<'p>, String> {
    if let Some((id, scope)) = declared.get_key_value(name) {
        return Ok(Resolved::Attribute(id, *scope));
    }
    if name == TARGET {
        return Ok(Resolved::Attribute(TARGET, Scope::Request));
    }
    if fhir_environment_variable(name).is_some() {
        return Ok(Resolved::Environment);
    }
    Err(format!("unknown attribute '{name}'"))
}

/// Request-scope values exist only while a decision rule's condition runs, a constraint
/// substitutes principal values alone, and an attribute's operation depends on built-ins
/// alone.
fn allowed(site: Site, resolved: Resolved<'_>) -> Result<(), String> {
    match (site, resolved) {
        (Site::Target, Resolved::Attribute(id, Scope::Request)) => Err(format!(
            "a rule target cannot use the request-scope attribute '{id}'"
        )),
        (Site::Data, Resolved::Attribute(id, Scope::Request)) => Err(format!(
            "a constraint cannot use the request-scope attribute '{id}'"
        )),
        (Site::Data, Resolved::Request) => {
            Err("a constraint can only substitute %principal values, not %request".to_string())
        }
        (Site::Operation(_), Resolved::Attribute(id, _)) => Err(format!(
            "an attribute cannot depend on the attribute '{id}'"
        )),
        (Site::Operation(Scope::Principal), Resolved::Request) => {
            Err("a principal-scope attribute cannot use %request".to_string())
        }
        _ => Ok(()),
    }
}

const fn site_name(site: Site) -> &'static str {
    match site {
        Site::Target => "target",
        Site::Condition => "condition",
        Site::Data => "data",
        Site::Operation(_) => "operation",
    }
}

/// The `%principal.x` and `%x` substitutions of a search string, in order. A `%` not
/// followed by an identifier is left alone.
fn substitutions(search: &str) -> impl Iterator<Item = ExternalConstant> + '_ {
    search.match_indices('%').filter_map(move |(at, _)| {
        let rest = &search[at + 1..];
        let name = identifier(rest)?;
        let member = rest[name.len()..]
            .strip_prefix('.')
            .and_then(identifier)
            .map(str::to_string);
        Some(ExternalConstant {
            name: name.to_string(),
            member,
        })
    })
}

/// The identifier `text` starts with, if any.
fn identifier(text: &str) -> Option<&str> {
    let starts = |c: char| c.is_ascii_alphabetic() || c == '_';
    let continues = |c: char| c.is_ascii_alphanumeric() || c == '_';
    let end = text
        .char_indices()
        .take_while(|&(i, c)| if i == 0 { starts(c) } else { continues(c) })
        .last()
        .map(|(i, c)| i + c.len_utf8())?;
    Some(&text[..end])
}

fn manifest_entry(rule: &str, attributes: &[Attribute<'_>]) -> AccessPolicyV3Manifest {
    AccessPolicyV3Manifest {
        rule: Box::new(FHIRString {
            value: Some(rule.to_string()),
            ..Default::default()
        }),
        attribute: attributes
            .iter()
            .map(|(id, scope)| AccessPolicyV3ManifestAttribute {
                attributeId: Box::new(FHIRId {
                    value: Some((*id).to_string()),
                    ..Default::default()
                }),
                scope: match scope {
                    Scope::Principal => Accesspolicyv3attributescope::principal(),
                    Scope::Request => Accesspolicyv3attributescope::request(),
                },
            })
            .collect(),
    }
}
