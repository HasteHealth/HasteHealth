//! Resolving a search parameter's `FHIRPath` expression against the
//! `StructureDefinition` snapshots, to learn whether it can yield more than
//! one value.
//!
//! A parameter that yields at most one value can be stored as a scalar column,
//! which is what lets an index answer an ordered comparison, a prefix match or
//! a sort. One that can yield more cannot.
//!
//! This answers the question for *plain dotted paths* — `Patient.birthDate`,
//! `Patient.name.family` — which is 86% of the HL7 base parameters. It reads
//! the schema rather than running anything, so the answer is exact and needs
//! no sample data. Expressions carrying `where()`, `ofType()`, a union or an
//! index accessor are reported as [`PathAnalysis::NotAPlainPath`] and belong to
//! the measuring analyzer in `haste-fhirpath`, which runs them.
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
    /// The FHIR type of the element the path ends on, when the element
    /// declares exactly one. `None` for a choice element or a backbone.
    pub leaf_type: Option<String>,
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

/// Walks `expression` through the snapshots, segment by segment.
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
        leaf_type: sole_type(current).map(ToString::to_string),
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

/// Whether a parameter produces at most one index entry per resource.
///
/// Both halves have to hold: the path selects at most one value, *and* that
/// value converts to at most one index entry. Anything the walker could not
/// resolve, or that needs the engine, is not single — the cost of being wrong
/// that way is slower storage, and the cost of being wrong the other way is
/// silently dropping values.
#[must_use]
pub fn is_single_valued(index: &SnapshotIndex, expression: &str) -> bool {
    match analyze_path(index, expression) {
        PathAnalysis::Resolved(path) => {
            !path.repeats
                && !path
                    .leaf_type
                    .as_deref()
                    .is_some_and(|leaf| FANNING_OUT_TYPES.contains(&leaf))
        }
        PathAnalysis::Unresolved { .. } | PathAnalysis::NotAPlainPath => false,
    }
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

    let mut urls: Vec<&str> = search_parameters
        .iter()
        .filter_map(|parameter| {
            let url = parameter.url.value.as_deref()?;
            let expression = parameter.expression.as_ref()?.value.as_deref()?;

            is_single_valued(&index, expression).then_some(url)
        })
        .collect();

    urls.sort_unstable();
    urls.dedup();

    let entries = urls.iter().fold(String::new(), |mut entries, url| {
        let _ = writeln!(entries, "    {url:?},");
        entries
    });

    format!(
        r#"//! Search parameters that produce at most one index value per resource.
//!
//! @generated by `bash scripts/search_param_cardinality_build.sh` — do not edit.
//!
//! A parameter listed here selects at most one value and converts to at most
//! one index entry, so it can be stored as a scalar column, which is what lets
//! an index answer an ordered comparison, a prefix match or a sort.
//!
//! Absence means "not known to be single". A parameter whose expression needs
//! the `FHIRPath` engine to resolve, or that the schema walk could not follow,
//! is absent for the same reason a genuinely repeating one is: storing several
//! values in a scalar column keeps the first and drops the rest.

/// Canonical URLs of the single-valued parameters, sorted for binary search.
static SINGLE_VALUED: [&str; {count}] = [
{entries}];

/// Whether `url` names a parameter that produces at most one index value.
///
/// Unknown URLs answer `false`, which is the safe direction: a caller that
/// treats an unclassified parameter as multi valued is slower, one that treats
/// it as single loses data.
#[must_use]
pub fn is_single_valued(url: &str) -> bool {{
    SINGLE_VALUED.binary_search(&url).is_ok()
}}
"#,
        count = urls.len(),
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

    fn index() -> SnapshotIndex<'static> {
        SnapshotIndex::new(DEFINITIONS.iter())
    }

    fn resolved(expression: &str) -> ResolvedPath {
        match analyze_path(&index(), expression) {
            PathAnalysis::Resolved(resolved) => resolved,
            other => panic!("{expression} did not resolve: {other:?}"),
        }
    }

    #[test]
    fn a_singular_element_does_not_repeat() {
        let birth_date = resolved("Patient.birthDate");

        assert!(!birth_date.repeats);
        assert_eq!(birth_date.leaf_type.as_deref(), Some("date"));
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
        assert_eq!(
            resolved("Patient.name.family").leaf_type.as_deref(),
            Some("string")
        );
        // Two crossings: Patient.contact is a backbone, its name a HumanName.
        assert!(resolved("Patient.contact.name.family").repeats);
    }

    /// Cardinality is not fan-out. This selects one CodeableConcept, which
    /// `indexing_conversion` turns into one token per coding — the caller has
    /// to combine the two, which is why the leaf type is reported.
    #[test]
    fn a_singular_codeable_concept_still_reports_its_type() {
        let code = resolved("Observation.code");

        assert!(!code.repeats, "Observation.code is 1..1");
        assert_eq!(code.leaf_type.as_deref(), Some("CodeableConcept"));
    }

    #[test]
    fn a_singular_reference_resolves() {
        let subject = resolved("Observation.subject");

        assert!(!subject.repeats);
        assert_eq!(subject.leaf_type.as_deref(), Some("Reference"));
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
    fn expressions_needing_the_engine_are_declined() {
        for expression in [
            "Patient.name.where(use='official')",
            "Patient.deceased.ofType(dateTime)",
            "Patient.birthDate | Patient.deceased",
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

    /// The committed table is generated from these same artifacts, so it has
    /// to match what the generator produces now. Without this, a change to the
    /// walker, the fan-out list or the HL7 package silently leaves a stale
    /// table behind — and a stale table is one that may call a parameter
    /// single-valued when it is not.
    #[test]
    fn the_committed_table_is_up_to_date() {
        let expected = generate_lookup(&DEFINITIONS, &SEARCH_PARAMETERS);
        let committed = include_str!("../../fhir-search/src/search_parameter_cardinality.rs");

        // The committed file has been through rustfmt; compare on content
        // rather than layout.
        let normalize = |source: &str| {
            source
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>()
                .join("\n")
        };

        assert_eq!(
            normalize(&expected),
            normalize(committed),
            "run `bash scripts/search_param_cardinality_build.sh`",
        );
    }

    /// A parameter selecting one `CodeableConcept` is not single valued, even
    /// though its path does not repeat, because the conversion fans it out.
    #[test]
    fn fanning_out_types_are_not_single_valued() {
        let index = index();

        assert!(!is_single_valued(&index, "Observation.code"));
        assert!(is_single_valued(&index, "Observation.subject"));
        assert!(is_single_valued(&index, "Patient.birthDate"));
    }

    /// Runs the whole HL7 base corpus, so a change in the walker or the
    /// artifacts shows up as a shift in these counts rather than silently
    /// reclassifying parameters.
    #[test]
    fn the_base_corpus_classifies_stably() {
        let index = index();
        let (mut single, mut many, mut not_plain, mut unresolved) = (0, 0, 0, 0);

        for parameter in SEARCH_PARAMETERS.iter() {
            let Some(expression) = parameter
                .expression
                .as_ref()
                .and_then(|e| e.value.as_deref())
            else {
                continue;
            };

            match analyze_path(&index, expression) {
                PathAnalysis::Resolved(path) if path.repeats => many += 1,
                PathAnalysis::Resolved(_) => single += 1,
                PathAnalysis::NotAPlainPath => not_plain += 1,
                PathAnalysis::Unresolved { .. } => unresolved += 1,
            }
        }

        let total = single + many + not_plain + unresolved;
        assert_eq!(total, 1372, "corpus size");

        // Every plain path the walker declines to resolve is a parameter that
        // falls back to the slower storage, so the count is worth watching.
        assert!(
            unresolved <= 15,
            "unresolved plain paths grew: {unresolved}",
        );
        assert!(
            single >= 600,
            "singular paths shrank to {single}, which shrinks the scalar-column win",
        );

        println!("single={single} many={many} not_plain={not_plain} unresolved={unresolved}");
    }
}
