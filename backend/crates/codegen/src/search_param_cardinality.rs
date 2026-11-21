//! Resolving a search parameter's `FHIRPath` expression against the
//! `StructureDefinition` snapshots, to learn whether it can yield more than
//! one value.
//!
//! A parameter that yields at most one value can be stored as a scalar column,
//! which is what lets an index answer an ordered comparison, a prefix match or
//! a sort. One that can yield more cannot.
//!
//! The answer is given per (parameter, resource type), because most shared
//! base parameters are a union with one branch per type —
//! `AllergyIntolerance.patient | … | Observation.subject.where(resolve() is
//! Patient) | …` — and a resource only ever takes its own branch. Judging the
//! union as a whole would let one repeating branch (`DocumentReference`'s
//! encounter) deny a column to every other type.
//!
//! Each branch is reduced to a *plain dotted path* — `Patient.birthDate`,
//! `Patient.name.family` — and resolved against the schema, so the answer is
//! exact and needs no sample data. A `where()` filter is dropped from the path,
//! because a filter can only remove values; an `ofType()` or `as` cast narrows
//! the leaf to the cast's type. Anything else — an index accessor, a function
//! the walk cannot see through — leaves the branch unanswered, and the
//! parameter repeating for that type.
//!
//! Note what this deliberately does not answer. Cardinality is the count of
//! *values the expression selects*, not the count of *index entries they
//! become*: `Observation.code` selects one `CodeableConcept`, which fans out to
//! one token per coding. Callers have to combine `repeats` with the leaf type's
//! fan-out, which is why [`ResolvedPath::leaf_type`] is reported alongside.

use std::collections::HashMap;
use std::fmt::Write as _;

use haste_fhir_model::r4::generated::{resources::StructureDefinition, types::ElementDefinition};

use crate::utilities::extract::{self, Max};

/// What walking a path against the snapshots established.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathAnalysis {
    Resolved(ResolvedPath),
    /// A segment had no matching element. Reported rather than guessed at: a
    /// path the walker cannot follow must not be assumed singular.
    Unresolved {
        /// The element path reached before the walk failed.
        reached: String,
        /// The segment that could not be found under it.
        segment: String,
    },
    /// The expression is not a plain dotted path, so the schema alone cannot
    /// answer it.
    NotAPlainPath,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPath {
    /// Whether any element along the path may occur more than once. True for
    /// `Patient.name.family`, because `name` repeats even though `family`
    /// does not.
    pub repeats: bool,
    /// Every FHIR type the element the path ends on may take: one for most
    /// elements, several for a choice element (`Observation.effective[x]`).
    pub leaf_types: Vec<String>,
}

/// The snapshots a path is resolved against, keyed by the type they define.
pub struct SnapshotIndex<'a> {
    by_type: HashMap<&'a str, &'a StructureDefinition>,
}

impl<'a> SnapshotIndex<'a> {
    /// Indexes definitions by their `type`, ignoring any without a snapshot —
    /// a differential alone cannot be walked.
    #[must_use]
    pub fn new(definitions: impl IntoIterator<Item = &'a StructureDefinition>) -> Self {
        let mut by_type = HashMap::new();

        for sd in definitions {
            if sd.snapshot.is_none() {
                continue;
            }
            if let Some(type_name) = sd.type_.value.as_deref() {
                by_type.insert(type_name, sd);
            }
        }

        Self { by_type }
    }

    fn elements(&self, type_name: &str) -> Option<&'a [ElementDefinition]> {
        self.by_type
            .get(type_name)?
            .snapshot
            .as_ref()
            .map(|snapshot| snapshot.element.as_slice())
    }

    /// Finds the element at exactly `path` within the definition of the type
    /// that path starts with.
    fn element_at(&self, path: &str) -> Option<&'a ElementDefinition> {
        let type_name = path.split('.').next()?;

        self.elements(type_name)?
            .iter()
            .find(|element| element.path.value.as_deref() == Some(path))
    }
}

/// Whether every character is one a plain dotted path can contain. Anything
/// else — a call, a union, an index — means the schema alone cannot answer the
/// question.
fn is_plain_path(expression: &str) -> bool {
    !expression.is_empty()
        && expression
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.')
        && !expression.starts_with('.')
        && !expression.ends_with('.')
}

fn repeats(element: &ElementDefinition) -> bool {
    !matches!(extract::cardinality(element).1, Max::Fixed(1))
}

/// The single type an element declares, if it declares exactly one. A choice
/// element declares several and a backbone declares `BackboneElement`, neither
/// of which identifies where the walk continues on its own.
fn sole_type(element: &ElementDefinition) -> Option<&str> {
    match extract::field_types(element).as_slice() {
        [single] => Some(single),
        _ => None,
    }
}

/// Walks a plain dotted path through the snapshots, segment by segment.
///
/// The walk crosses definitions: once a path leaves the resource's own
/// elements — `Patient.name` is a `HumanName` — it continues in the definition
/// of that type. `contentReference` is followed the same way, which is what
/// lets a recursive structure like `Questionnaire.item.item` resolve.
#[must_use]
pub fn analyze_path(index: &SnapshotIndex, expression: &str) -> PathAnalysis {
    if !is_plain_path(expression) {
        return PathAnalysis::NotAPlainPath;
    }

    let mut segments = expression.split('.');

    // The first segment names the type the walk starts in.
    let Some(root) = segments.next() else {
        return PathAnalysis::NotAPlainPath;
    };

    let Some(root_element) = index.element_at(root) else {
        return PathAnalysis::Unresolved {
            reached: String::new(),
            segment: root.to_string(),
        };
    };

    let mut current = root_element;
    let mut current_path = root.to_string();
    let mut saw_repeat = false;

    for segment in segments {
        let Some(next) = step(index, current, &current_path, segment) else {
            return PathAnalysis::Unresolved {
                reached: current_path,
                segment: segment.to_string(),
            };
        };

        saw_repeat |= repeats(next.element);
        current = next.element;
        current_path = next.path;
    }

    PathAnalysis::Resolved(ResolvedPath {
        repeats: saw_repeat,
        leaf_types: extract::field_types(current)
            .into_iter()
            .map(ToString::to_string)
            .collect(),
    })
}

struct Step<'a> {
    element: &'a ElementDefinition,
    path: String,
}

/// Resolves one segment below `current`, following into another definition or
/// a content reference when the element is not defined inline.
fn step<'a>(
    index: &SnapshotIndex<'a>,
    current: &'a ElementDefinition,
    current_path: &str,
    segment: &str,
) -> Option<Step<'a>> {
    // Defined inline, as a backbone element's children are.
    let inline = format!("{current_path}.{segment}");
    if let Some(element) = index.element_at(&inline) {
        return Some(Step {
            element,
            path: inline,
        });
    }

    // A choice element is written `value[x]` but named `value` in a path.
    let choice = format!("{current_path}.{segment}[x]");
    if let Some(element) = index.element_at(&choice) {
        return Some(Step {
            element,
            path: choice,
        });
    }

    // Recursive structures carry their children by reference rather than
    // repeating them, so the walk continues wherever the reference points.
    if let Some(target) = current
        .contentReference
        .as_ref()
        .and_then(|r| r.value.as_deref())
        .and_then(|r| r.strip_prefix('#'))
    {
        let referenced = format!("{target}.{segment}");
        if let Some(element) = index.element_at(&referenced) {
            return Some(Step {
                element,
                path: referenced,
            });
        }
    }

    // Otherwise the path has left this definition and continues in the one for
    // the element's own type.
    let type_name = sole_type(current)?;
    let in_type = format!("{type_name}.{segment}");
    index.element_at(&in_type).map(|element| Step {
        element,
        path: in_type,
    })
}

/// Datatypes whose conversion to index values emits more than one entry for a
/// single value, so a path that selects exactly one of them still produces
/// several index entries.
///
/// This mirrors the per-type arms of `indexing_conversion` in
/// `haste-fhir-search`: a `HumanName` becomes its text, family, every given,
/// every prefix and every suffix; a `CodeableConcept` becomes one token per
/// coding. Changing a converter to fan out — encoding an `Identifier` as both
/// `system|value` and bare `value`, say — means adding its type here, or the
/// generated table starts claiming parameters are singular when their values
/// are being dropped.
pub const FANNING_OUT_TYPES: [&str; 4] = ["HumanName", "Address", "CodeableConcept", "Timing"];

/// One union branch reduced to what the schema can answer: the path it walks,
/// and the type a cast narrows its leaf to.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Branch {
    path: String,
    cast: Option<String>,
}

/// The resource type a branch starts from, read off its text so that even a
/// branch that cannot be reduced is still known to apply, or not, to a type.
fn branch_root(branch: &str) -> &str {
    let branch = branch.trim_start_matches(|c: char| c == '(' || c.is_whitespace());
    let end = branch
        .find(|c: char| !c.is_ascii_alphanumeric())
        .unwrap_or(branch.len());
    &branch[..end]
}

/// Splits `expression` on the `|` operators at its top level, leaving any
/// inside parentheses — a `where()` argument's — alone.
fn split_union(expression: &str) -> Vec<&str> {
    let mut branches = Vec::new();
    let mut depth = 0usize;
    let mut start = 0;

    for (at, c) in expression.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            '|' if depth == 0 => {
                branches.push(expression[start..at].trim());
                start = at + 1;
            }
            _ => {}
        }
    }
    branches.push(expression[start..].trim());
    branches
}

/// Whether `text` is one parenthesised group from its first character to its
/// last, so the outer pair can be dropped without changing its meaning.
fn is_wrapped(text: &str) -> bool {
    if !text.starts_with('(') || !text.ends_with(')') {
        return false;
    }
    let mut depth = 0usize;
    for (at, c) in text.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 && at != text.len() - 1 {
                    return false;
                }
            }
            _ => {}
        }
    }
    true
}

/// Removes every `.where(...)` from `path`. A filter keeps a subset of what it
/// is given, so the path without it selects at least as many values — which
/// makes the result a safe bound for the question asked of it.
fn strip_where(path: &str) -> Option<String> {
    let mut out = String::with_capacity(path.len());
    let mut rest = path;

    while let Some(at) = rest.find(".where(") {
        out.push_str(&rest[..at]);
        let mut depth = 0usize;
        let mut close = None;
        for (offset, c) in rest[at + ".where".len()..].char_indices() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        close = Some(at + ".where".len() + offset);
                        break;
                    }
                }
                _ => {}
            }
        }
        rest = &rest[close? + 1..];
    }

    out.push_str(rest);
    Some(out)
}

/// Reduces one union branch to a plain path and an optional cast, or `None`
/// when it uses anything the schema walk cannot see through.
fn simplify_branch(branch: &str) -> Option<Branch> {
    let mut text = branch.trim();
    while is_wrapped(text) {
        text = text[1..text.len() - 1].trim();
    }

    // `(Observation.value as Quantity)`
    let (text, cast) = match text.split_once(" as ") {
        Some((path, cast)) => (path.trim(), Some(cast.trim())),
        None => (text, None),
    };

    // `RiskAssessment.occurrence.ofType(dateTime)`
    let (text, cast) = match text
        .strip_suffix(')')
        .and_then(|t| t.rsplit_once(".ofType("))
    {
        Some((path, of_type)) if cast.is_none() => (path, Some(of_type)),
        Some(_) => return None,
        None => (text, cast),
    };

    let path = strip_where(text)?;
    if !is_plain_path(&path) || cast.is_some_and(|cast| !is_plain_path(cast)) {
        return None;
    }

    Some(Branch {
        path,
        cast: cast.map(ToString::to_string),
    })
}

/// Bases whose parameters apply to every resource type.
const UNIVERSAL_BASES: [&str; 2] = ["Resource", "DomainResource"];

/// Whether a parameter produces at most one index entry for a resource of
/// type `base`.
///
/// Only the branches a `base` resource can take count: those rooted at it,
/// and at the universal bases. Every one of them has to reduce to the same
/// path — several branches over one choice element, each casting it to a
/// different type, still select at most one value, because the element holds
/// one type at a time — and that path must not repeat.
///
/// And the value has to convert to at most one index entry, for *every* type
/// it may take: a choice element that can hold a `Timing` fans out whenever
/// it does.
///
/// Anything the walker could not resolve, or that needs the engine, is not
/// single — the cost of being wrong that way is slower storage, and the cost
/// of being wrong the other way is silently dropping values.
#[must_use]
pub fn is_single_valued_for(index: &SnapshotIndex, expression: &str, base: &str) -> bool {
    let mut applicable = Vec::new();

    for branch in split_union(expression) {
        let root = branch_root(branch);
        if root != base && !UNIVERSAL_BASES.contains(&root) {
            continue;
        }

        let Some(simplified) = simplify_branch(branch) else {
            return false;
        };
        applicable.push(simplified);
    }

    let Some(first) = applicable.first() else {
        return false;
    };
    if applicable.iter().any(|branch| branch.path != first.path) {
        return false;
    }

    let PathAnalysis::Resolved(resolved) = analyze_path(index, &first.path) else {
        return false;
    };
    if resolved.repeats {
        return false;
    }

    // A cast narrows the leaf to its type; an uncast branch can yield any
    // type the element declares.
    let mut leaf_types: Vec<&str> = Vec::new();
    for branch in &applicable {
        match &branch.cast {
            Some(cast) => leaf_types.push(cast),
            None => leaf_types.extend(resolved.leaf_types.iter().map(String::as_str)),
        }
    }

    !leaf_types.is_empty()
        && !leaf_types
            .iter()
            .any(|leaf| FANNING_OUT_TYPES.contains(leaf))
}

/// Reads every `StructureDefinition` under the given files or directories,
/// following the same JSON-file walk the other generators use. Bundles are
/// unwrapped, so a `profiles-resources.min.json` can be passed directly.
///
/// # Errors
///
/// Returns an error if a path cannot be read or a file is not valid JSON.
pub fn load_definitions(paths: &[String]) -> Result<Vec<StructureDefinition>, String> {
    load_resources(paths, |resource| match resource {
        haste_fhir_model::r4::generated::resources::Resource::StructureDefinition(sd) => Some(sd),
        _ => None,
    })
}

/// Reads every `SearchParameter` under the given files or directories.
///
/// # Errors
///
/// Returns an error if a path cannot be read or a file is not valid JSON.
pub fn load_search_parameters(
    paths: &[String],
) -> Result<Vec<haste_fhir_model::r4::generated::resources::SearchParameter>, String> {
    load_resources(paths, |resource| match resource {
        haste_fhir_model::r4::generated::resources::Resource::SearchParameter(sp) => Some(sp),
        _ => None,
    })
}

fn load_resources<T>(
    paths: &[String],
    pick: impl Fn(haste_fhir_model::r4::generated::resources::Resource) -> Option<T> + Copy,
) -> Result<Vec<T>, String> {
    use haste_fhir_model::r4::generated::resources::Resource;

    let mut collected = Vec::new();

    for path in paths {
        for entry in walkdir::WalkDir::new(path)
            .sort_by_file_name()
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.metadata().is_ok_and(|m| m.is_file()))
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
        {
            let contents = std::fs::read_to_string(entry.path())
                .map_err(|e| format!("{}: {e}", entry.path().display()))?;

            let resource: Resource = serde_json::from_str(&contents)
                .map_err(|e| format!("{}: {e}", entry.path().display()))?;

            match resource {
                // A bundle of definitions, as the HL7 packages ship them.
                Resource::Bundle(bundle) => {
                    collected.extend(
                        bundle
                            .entry
                            .unwrap_or_default()
                            .into_iter()
                            .filter_map(|e| e.resource)
                            .filter_map(|r| pick(*r)),
                    );
                }
                resource => collected.extend(pick(resource)),
            }
        }
    }

    Ok(collected)
}

/// Emits the Rust source for the compiled lookup table.
///
/// The table is the sorted set of canonical URLs whose parameters are single
/// valued, so a lookup is a binary search over static data with no
/// initialisation. Absence means "not known to be single", which is the answer
/// a caller should act on anyway for a URL it has never heard of.
#[must_use]
pub fn generate_lookup(
    definitions: &[StructureDefinition],
    search_parameters: &[haste_fhir_model::r4::generated::resources::SearchParameter],
) -> String {
    let index = SnapshotIndex::new(definitions.iter());
    let index = &index;

    let mut pairs: Vec<(&str, &str)> = search_parameters
        .iter()
        .flat_map(|parameter| {
            let url = parameter.url.value.as_deref();
            let expression = parameter
                .expression
                .as_ref()
                .and_then(|e| e.value.as_deref());

            parameter.base.iter().filter_map(move |base| {
                let (url, expression, base) = (url?, expression?, base.as_str()?);
                is_single_valued_for(index, expression, base).then_some((url, base))
            })
        })
        .collect();

    pairs.sort_unstable();
    pairs.dedup();

    let entries = pairs
        .iter()
        .fold(String::new(), |mut entries, (url, base)| {
            let _ = writeln!(entries, "    ({url:?}, {base:?}),");
            entries
        });

    format!(
        r#"//! Search parameters that produce at most one index value per resource.
//!
//! @generated by `bash scripts/search_param_cardinality_build.sh` — do not edit.
//!
//! A parameter listed here for a resource type selects at most one value from
//! a resource of that type, and converts it to at most one index entry, so it
//! can be stored as a scalar column, which is what lets an index answer an
//! ordered comparison, a prefix match or a sort. The answer is per type:
//! a shared parameter like `clinical-patient` takes one branch of its union
//! per type, and can be single for some and repeating for others.
//!
//! Absence means "not known to be single". A parameter whose expression needs
//! the `FHIRPath` engine to resolve, or that the schema walk could not follow,
//! is absent for the same reason a genuinely repeating one is: storing several
//! values in a scalar column keeps the first and drops the rest.

/// (canonical URL, base) pairs of the single-valued parameters, sorted for
/// binary search.
static SINGLE_VALUED: [(&str, &str); {count}] = [
{entries}];

/// Whether the parameter `url` produces at most one index value for a
/// resource of type `resource_type` — as a parameter of that type, or of
/// every type (`Resource`, `DomainResource`).
///
/// Unknown pairs answer `false`, which is the safe direction: a caller that
/// treats an unclassified parameter as multi valued is slower, one that treats
/// it as single loses data.
#[must_use]
pub fn is_single_valued(url: &str, resource_type: &str) -> bool {{
    let listed = |base: &str| {{
        SINGLE_VALUED
            .binary_search_by(|(u, b)| (*u, *b).cmp(&(url, base)))
            .is_ok()
    }};

    listed(resource_type) || listed("Resource") || listed("DomainResource")
}}
"#,
        count = pairs.len(),
        entries = entries,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use haste_fhir_model::r4::generated::resources::{Bundle, Resource, SearchParameter};
    use std::sync::LazyLock;

    fn definitions_from(json: &str) -> Vec<StructureDefinition> {
        serde_json::from_str::<Bundle>(json)
            .expect("bundle parses")
            .entry
            .unwrap_or_default()
            .into_iter()
            .filter_map(|e| e.resource)
            .filter_map(|r| match *r {
                Resource::StructureDefinition(sd) => Some(sd),
                _ => None,
            })
            .collect()
    }

    static DEFINITIONS: LazyLock<Vec<StructureDefinition>> = LazyLock::new(|| {
        let mut all = definitions_from(include_str!(
            "../../../../artifacts/r4/hl7-core/definitions/hl7/profiles-resources.min.json"
        ));
        all.extend(definitions_from(include_str!(
            "../../../../artifacts/r4/hl7-core/definitions/hl7/profiles-types.min.json"
        )));
        all
    });

    static SEARCH_PARAMETERS: LazyLock<Vec<SearchParameter>> = LazyLock::new(|| {
        serde_json::from_str::<Bundle>(include_str!(
            "../../../../artifacts/r4/hl7-core/definitions/hl7/search-parameters.min.json"
        ))
        .expect("bundle parses")
        .entry
        .unwrap_or_default()
        .into_iter()
        .filter_map(|e| e.resource)
        .filter_map(|r| match *r {
            Resource::SearchParameter(sp) => Some(sp),
            _ => None,
        })
        .collect()
    });

    /// The single (parameter, type) pairs in the HL7 base corpus. Below this,
    /// something stopped resolving.
    const SINGLE_PAIRS_FLOOR: usize = 780;

    fn index() -> SnapshotIndex<'static> {
        SnapshotIndex::new(DEFINITIONS.iter())
    }

    fn resolved(expression: &str) -> ResolvedPath {
        match analyze_path(&index(), expression) {
            PathAnalysis::Resolved(resolved) => resolved,
            other => panic!("{expression} did not resolve: {other:?}"),
        }
    }

    fn types(resolved: &ResolvedPath) -> Vec<&str> {
        resolved.leaf_types.iter().map(String::as_str).collect()
    }

    #[test]
    fn a_singular_element_does_not_repeat() {
        let birth_date = resolved("Patient.birthDate");

        assert!(!birth_date.repeats);
        assert_eq!(types(&birth_date), ["date"]);
    }

    /// `family` is `0..1`, but `name` above it is `0..*`, so the path repeats.
    #[test]
    fn a_repeating_element_anywhere_on_the_path_repeats() {
        assert!(resolved("Patient.name.family").repeats);
        assert!(resolved("Patient.name").repeats);
    }

    /// The walk crosses into another definition when the path leaves the
    /// resource's own elements: `name` is a `HumanName`, `family` lives there.
    #[test]
    fn the_walk_crosses_into_complex_types() {
        assert_eq!(types(&resolved("Patient.name.family")), ["string"]);
        // Two crossings: Patient.contact is a backbone, its name a HumanName.
        assert!(resolved("Patient.contact.name.family").repeats);
    }

    /// Cardinality is not fan-out. This selects one CodeableConcept, which
    /// `indexing_conversion` turns into one token per coding — the caller has
    /// to combine the two, which is why the leaf types are reported.
    #[test]
    fn a_singular_codeable_concept_still_reports_its_type() {
        let code = resolved("Observation.code");

        assert!(!code.repeats, "Observation.code is 1..1");
        assert_eq!(types(&code), ["CodeableConcept"]);
    }

    /// A choice element reports every type it may hold.
    #[test]
    fn a_choice_element_reports_all_its_types() {
        let effective = resolved("Observation.effective");

        assert!(!effective.repeats);
        assert!(types(&effective).contains(&"Timing"));
        assert!(types(&effective).contains(&"dateTime"));
    }

    #[test]
    fn a_singular_reference_resolves() {
        let subject = resolved("Observation.subject");

        assert!(!subject.repeats);
        assert_eq!(types(&subject), ["Reference"]);
    }

    /// A recursive structure carries its children by `contentReference`.
    #[test]
    fn content_references_are_followed() {
        assert!(resolved("Questionnaire.item.item.text").repeats);
    }

    #[test]
    fn an_unknown_segment_is_reported_not_guessed() {
        assert_eq!(
            analyze_path(&index(), "Patient.notAnElement"),
            PathAnalysis::Unresolved {
                reached: "Patient".to_string(),
                segment: "notAnElement".to_string(),
            }
        );
    }

    #[test]
    fn expressions_needing_the_engine_are_not_plain_paths() {
        for expression in [
            "Patient.name.where(use='official')",
            "Patient.deceased.ofType(dateTime)",
            "(Observation.value as Quantity)",
            "Patient.extension[0]",
        ] {
            assert_eq!(
                analyze_path(&index(), expression),
                PathAnalysis::NotAPlainPath,
                "{expression}",
            );
        }
    }

    #[test]
    fn branches_reduce_to_a_path_and_a_cast() {
        let branch = |path: &str, cast: Option<&str>| Branch {
            path: path.to_string(),
            cast: cast.map(ToString::to_string),
        };

        assert_eq!(
            simplify_branch("Observation.subject.where(resolve() is Patient)"),
            Some(branch("Observation.subject", None))
        );
        assert_eq!(
            simplify_branch("(RiskAssessment.occurrence.ofType(dateTime))"),
            Some(branch("RiskAssessment.occurrence", Some("dateTime")))
        );
        assert_eq!(
            simplify_branch("(Observation.value as Quantity)"),
            Some(branch("Observation.value", Some("Quantity")))
        );
        assert_eq!(simplify_branch("Patient.extension[0]"), None);
        assert_eq!(simplify_branch("Patient.name.first()"), None);
    }

    /// A `|` inside a filter's argument is not a branch boundary.
    #[test]
    fn the_union_splits_only_at_the_top_level() {
        assert_eq!(
            split_union("A.b.where(c | d) | E.f"),
            ["A.b.where(c | d)", "E.f"]
        );
    }

    fn single(expression: &str, base: &str) -> bool {
        is_single_valued_for(&index(), expression, base)
    }

    /// Each resource type takes only its own branch of a shared parameter.
    #[test]
    fn a_union_is_judged_per_resource_type() {
        let birthdate = "Patient.birthDate | Person.birthDate | RelatedPerson.birthDate";
        assert!(single(birthdate, "Patient"));
        assert!(single(birthdate, "Person"));

        // `DocumentReference.context.encounter` repeats; the others do not,
        // and one type's repetition no longer costs every other type its
        // column.
        let encounter = "DocumentReference.context.encounter | Observation.encounter";
        assert!(!single(encounter, "DocumentReference"));
        assert!(single(encounter, "Observation"));
    }

    /// `clinical-patient` filters each type's subject down to patients; a
    /// filter only removes values, so the subject's cardinality bounds it.
    #[test]
    fn a_filtered_branch_is_as_single_as_its_path() {
        assert!(single(
            "AllergyIntolerance.patient | Observation.subject.where(resolve() is Patient)",
            "Observation"
        ));
        assert!(single(
            "AllergyIntolerance.patient | Observation.subject.where(resolve() is Patient)",
            "AllergyIntolerance"
        ));
    }

    /// A choice element that may hold a fanning-out type is not single, even
    /// though it holds one value: when that value is a `Timing` it becomes
    /// several dates. A cast to a type that does not fan out is single.
    #[test]
    fn a_choice_is_single_only_if_none_of_its_types_fans_out() {
        assert!(!single("Observation.effective", "Observation"));
        assert!(single(
            "(RiskAssessment.occurrence.ofType(dateTime))",
            "RiskAssessment"
        ));
        assert!(single("Encounter.period", "Encounter"));
    }

    /// Several branches over one choice element, each cast differently, are
    /// mutually exclusive; branches over different elements are not.
    #[test]
    fn branches_for_one_type_must_select_the_same_element() {
        assert!(single(
            "(Observation.value as Quantity) | (Observation.value as SampledData)",
            "Observation"
        ));
        assert!(!single(
            "Observation.issued | Observation.effective",
            "Observation"
        ));
    }

    /// A branch the walk cannot reduce leaves its own type unanswered, and
    /// only its own type.
    #[test]
    fn an_unreducible_branch_only_affects_its_type() {
        let expression = "Patient.extension[0] | Observation.subject";
        assert!(!single(expression, "Patient"));
        assert!(single(expression, "Observation"));
    }

    #[test]
    fn a_type_the_parameter_has_no_branch_for_is_not_single() {
        assert!(!single("Observation.subject", "Condition"));
    }

    /// A parameter selecting one `CodeableConcept` is not single valued, even
    /// though its path does not repeat, because the conversion fans it out.
    #[test]
    fn fanning_out_types_are_not_single_valued() {
        assert!(!single("Observation.code", "Observation"));
        assert!(single("Observation.subject", "Observation"));
        assert!(single("Patient.birthDate", "Patient"));
    }

    /// The shared clinical parameters, which is what the per-type analysis is
    /// for: each type now answers for its own branch.
    #[test]
    fn the_clinical_parameters_classify_per_type() {
        let expression = |id: &str| {
            SEARCH_PARAMETERS
                .iter()
                .find(|p| p.id.as_deref() == Some(id))
                .and_then(|p| p.expression.as_ref())
                .and_then(|e| e.value.clone())
                .unwrap_or_else(|| panic!("{id} has an expression"))
        };

        let patient = expression("clinical-patient");
        for base in [
            "Observation",
            "Condition",
            "Encounter",
            "Procedure",
            "AllergyIntolerance",
        ] {
            assert!(single(&patient, base), "clinical-patient on {base}");
        }

        let encounter = expression("clinical-encounter");
        assert!(single(&encounter, "Observation"));
        assert!(!single(&encounter, "DocumentReference"));

        let date = expression("clinical-date");
        assert!(single(&date, "Encounter"));
        assert!(
            !single(&date, "Observation"),
            "effective[x] may be a Timing"
        );

        let code = expression("clinical-code");
        assert!(!single(&code, "Observation"), "a CodeableConcept fans out");
    }

    /// Runs the whole HL7 base corpus per (parameter, type), so a change in
    /// the walker or the artifacts shows up as a shift in these counts rather
    /// than silently reclassifying parameters.
    #[test]
    fn the_base_corpus_classifies_stably() {
        let index = index();
        let (mut pairs, mut single) = (0, 0);

        for parameter in SEARCH_PARAMETERS.iter() {
            let Some(expression) = parameter
                .expression
                .as_ref()
                .and_then(|e| e.value.as_deref())
            else {
                continue;
            };

            for base in parameter.base.iter().filter_map(|b| b.as_str()) {
                pairs += 1;
                if is_single_valued_for(&index, expression, base) {
                    single += 1;
                }
            }
        }

        println!("single={single} of {pairs} (parameter, type) pairs");
        assert!(
            single >= SINGLE_PAIRS_FLOOR,
            "single pairs shrank to {single}, which shrinks the scalar-column win",
        );
    }
}
