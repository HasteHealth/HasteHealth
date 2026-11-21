use haste_fhir_client::canonical_resolver::CanonicalResolver;
use haste_fhir_model::r4::generated::resources::{OperationOutcomeIssue, Resource, ResourceType};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_profiling::{FHIRProfileArguments, validate_profile_by_url};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

const VITAL_SIGNS: &str = "http://hl7.org/fhir/us/core/StructureDefinition/us-core-vital-signs";
const US_CORE_PATIENT: &str = "http://hl7.org/fhir/us/core/StructureDefinition/us-core-patient";
const RACE: &str = "http://hl7.org/fhir/us/core/StructureDefinition/us-core-race";
const OBSERVATION_CATEGORY: &str = "http://terminology.hl7.org/CodeSystem/observation-category";

/// The US Core `StructureDefinition` resources, keyed by canonical URL.
static US_CORE: LazyLock<HashMap<String, Arc<Resource>>> = LazyLock::new(|| {
    let dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../artifacts/r4/hl7.fhir.us.core"
    );
    std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("StructureDefinition-"))
        })
        .map(|path| {
            let resource: Resource =
                serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
            let Resource::StructureDefinition(sd) = &resource else {
                panic!("Expected a StructureDefinition");
            };
            (sd.url.value.clone().unwrap(), Arc::new(resource))
        })
        .collect()
});

struct UsCore;

impl CanonicalResolver for UsCore {
    fn resolve(
        &self,
        _resource_type: ResourceType,
        canonical_url: &str,
    ) -> impl Future<Output = Result<Option<Arc<Resource>>, OperationOutcomeError>> + Send {
        std::future::ready(Ok(US_CORE.get(canonical_url).cloned()))
    }
}

/// Validates `resource` against `profile` and returns the issues as JSON.
async fn validate(profile: &str, resource: Value) -> Value {
    let resource: Resource = serde_json::from_value(resource).unwrap();
    let outcome = validate_profile_by_url(
        FHIRProfileArguments::new(Arc::new(UsCore)),
        profile,
        &resource,
    )
    .await
    .unwrap();
    serde_json::to_value(&outcome.issue).unwrap()
}

fn category(code: &str) -> Value {
    json!({"coding": [{"system": OBSERVATION_CATEGORY, "code": code}]})
}

fn vital_signs_observation(categories: &[&str]) -> Value {
    json!({
        "resourceType": "Observation",
        "status": "final",
        "category": categories.iter().map(|code| category(code)).collect::<Vec<_>>(),
        "code": {"coding": [{"system": "http://loinc.org", "code": "85354-9"}]},
        "subject": {"reference": "Patient/1"},
        "effectiveDateTime": "2024-01-01"
    })
}

fn race_extension(text: &str) -> Value {
    json!({
        "url": RACE,
        "extension": [{"url": "text", "valueString": text}]
    })
}

fn us_core_patient(extensions: &[Value]) -> Value {
    json!({
        "resourceType": "Patient",
        "extension": extensions,
        "identifier": [{"system": "urn:example", "value": "1"}],
        "name": [{"family": "Smith"}],
        "gender": "female"
    })
}

#[tokio::test]
async fn non_matching_category_leaves_the_required_slice_empty() {
    assert_eq!(
        validate(VITAL_SIGNS, vital_signs_observation(&["laboratory"])).await,
        json!([{
            "severity": "error",
            "code": "value",
            "diagnostics": "Profile: 'us-core-vital-signs' Element: 'Observation.category:VSCat' Minimum number of required values not met expected at least '1', found '0'"
        }])
    );
}

#[tokio::test]
async fn matching_category_fills_the_slice() {
    assert_eq!(
        validate(
            VITAL_SIGNS,
            vital_signs_observation(&["laboratory", "vital-signs", "imaging"])
        )
        .await,
        json!([])
    );
}

#[tokio::test]
async fn two_matching_categories_exceed_the_slice_maximum() {
    assert_eq!(
        validate(
            VITAL_SIGNS,
            vital_signs_observation(&["vital-signs", "laboratory", "vital-signs"])
        )
        .await,
        json!([{
            "severity": "error",
            "code": "value",
            "diagnostics": "Cardinality too high: expected at most '1', found '2'",
            "location": ["/category/0"]
        }])
    );
}

/// The `race` slice's element is typed with the us-core-race profile, so
/// finding its `url` discriminator resolves that profile.
#[tokio::test]
async fn two_race_extensions_exceed_the_profiled_slice_maximum() {
    assert_eq!(
        validate(
            US_CORE_PATIENT,
            us_core_patient(&[
                race_extension("Asian"),
                json!({"url": "http://example.org/other", "valueString": "other"}),
                race_extension("White"),
            ])
        )
        .await,
        json!([{
            "severity": "error",
            "code": "value",
            "diagnostics": "Cardinality too high: expected at most '1', found '2'",
            "location": ["/extension/0"]
        }])
    );
}

/// The compiled profile is cached by the identity of the `StructureDefinition`
/// the resolver hands out, so a replaced profile (a new `Arc`, as a new version
/// in a store would be) is compiled afresh and its constraints apply at once.
#[tokio::test]
async fn a_replaced_profile_is_recompiled() {
    use std::sync::Mutex;

    const URL: &str = "http://example.org/StructureDefinition/identified-patient";

    struct Replaceable(Mutex<Arc<Resource>>);

    impl CanonicalResolver for Replaceable {
        fn resolve(
            &self,
            _resource_type: ResourceType,
            _canonical_url: &str,
        ) -> impl Future<Output = Result<Option<Arc<Resource>>, OperationOutcomeError>> + Send
        {
            std::future::ready(Ok(Some(self.0.lock().unwrap().clone())))
        }
    }

    fn profile(identifier_min: u64) -> Arc<Resource> {
        Arc::new(
            serde_json::from_value(json!({
                "resourceType": "StructureDefinition",
                "id": "identified-patient",
                "url": URL,
                "name": "IdentifiedPatient",
                "status": "active",
                "kind": "resource",
                "abstract": false,
                "type": "Patient",
                "derivation": "constraint",
                "snapshot": {"element": [
                    {"id": "Patient", "path": "Patient", "min": 0, "max": "*"},
                    {"id": "Patient.identifier", "path": "Patient.identifier", "min": identifier_min, "max": "*", "type": [{"code": "Identifier"}]}
                ]}
            }))
            .unwrap(),
        )
    }

    async fn issues(resolver: &Arc<Replaceable>, patient: &Resource) -> usize {
        validate_profile_by_url(FHIRProfileArguments::new(resolver.clone()), URL, patient)
            .await
            .unwrap()
            .issue
            .len()
    }

    let resolver = Arc::new(Replaceable(Mutex::new(profile(0))));
    let patient: Resource =
        serde_json::from_value(json!({"resourceType": "Patient", "name": [{"family": "Smith"}]}))
            .unwrap();
    assert_eq!(issues(&resolver, &patient).await, 0);
    assert_eq!(
        issues(&resolver, &patient).await,
        0,
        "served from the cache"
    );

    *resolver.0.lock().unwrap() = profile(1);
    assert_eq!(
        issues(&resolver, &patient).await,
        1,
        "the replaced profile requires an identifier"
    );
}

/// A resolver over a fixed set of resources keyed by canonical URL, recording
/// the URLs it is asked for.
struct Fixed {
    resources: HashMap<String, Arc<Resource>>,
    resolved: std::sync::Mutex<Vec<String>>,
}

impl Fixed {
    fn new(resources: impl IntoIterator<Item = Value>) -> Arc<Self> {
        let resources = resources
            .into_iter()
            .map(|value| {
                let url = value["url"].as_str().unwrap().to_string();
                (url, Arc::new(serde_json::from_value(value).unwrap()))
            })
            .collect();
        Arc::new(Self {
            resources,
            resolved: std::sync::Mutex::new(Vec::new()),
        })
    }
}

impl CanonicalResolver for Fixed {
    fn resolve(
        &self,
        _resource_type: ResourceType,
        canonical_url: &str,
    ) -> impl Future<Output = Result<Option<Arc<Resource>>, OperationOutcomeError>> + Send {
        self.resolved
            .lock()
            .unwrap()
            .push(canonical_url.to_string());
        std::future::ready(Ok(self.resources.get(canonical_url).cloned()))
    }
}

/// The diagnostics of every issue, as the outcome lists them.
fn diagnostics(issues: &[OperationOutcomeIssue]) -> Vec<&str> {
    issues
        .iter()
        .filter_map(|issue| issue.diagnostics.as_deref())
        .filter_map(|text| text.value.as_deref())
        .collect()
}

fn structure_definition(url: &str, type_: &str, elements: &Value) -> Value {
    json!({
        "resourceType": "StructureDefinition",
        "id": url.rsplit('/').next().unwrap(),
        "url": url,
        "name": "Test",
        "status": "active",
        "kind": "resource",
        "abstract": false,
        "type": type_,
        "derivation": "constraint",
        "snapshot": {"element": elements}
    })
}

/// A profile whose root element is typed with the profile itself (as a
/// profile on `Patient.link.other` might be typed with its own Patient
/// profile) applies to a value once; its constraints still hold.
#[tokio::test]
async fn a_profile_typed_with_itself_is_applied_once() {
    const URL: &str = "http://example.org/StructureDefinition/linked-patient";
    let resolver = Fixed::new([structure_definition(
        URL,
        "Patient",
        &json!([
            {"id": "Patient", "path": "Patient", "min": 0, "max": "*",
             "type": [{"code": "Patient", "profile": [URL]}]},
            {"id": "Patient.identifier", "path": "Patient.identifier", "min": 1, "max": "*",
             "type": [{"code": "Identifier"}]}
        ]),
    )]);
    let patient: Resource =
        serde_json::from_value(json!({"resourceType": "Patient", "name": [{"family": "Smith"}]}))
            .unwrap();
    let outcome = validate_profile_by_url(FHIRProfileArguments::new(resolver), URL, &patient)
        .await
        .unwrap();
    let diagnostics = diagnostics(&outcome.issue);
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert!(
        diagnostics[0].contains("Patient.identifier"),
        "{diagnostics:?}"
    );
}

/// Two profiles typed with each other at the root are each applied once to
/// the same value, so both sets of constraints hold.
#[tokio::test]
async fn profiles_typed_with_each_other_both_apply() {
    const A: &str = "http://example.org/StructureDefinition/patient-a";
    const B: &str = "http://example.org/StructureDefinition/patient-b";
    let resolver = Fixed::new([
        structure_definition(
            A,
            "Patient",
            &json!([
                {"id": "Patient", "path": "Patient", "min": 0, "max": "*",
                 "type": [{"code": "Patient", "profile": [B]}]},
                {"id": "Patient.identifier", "path": "Patient.identifier", "min": 1, "max": "*",
                 "type": [{"code": "Identifier"}]}
            ]),
        ),
        structure_definition(
            B,
            "Patient",
            &json!([
                {"id": "Patient", "path": "Patient", "min": 0, "max": "*",
                 "type": [{"code": "Patient", "profile": [A]}]},
                {"id": "Patient.gender", "path": "Patient.gender", "min": 1, "max": "1",
                 "type": [{"code": "code"}]}
            ]),
        ),
    ]);
    let patient: Resource =
        serde_json::from_value(json!({"resourceType": "Patient", "name": [{"family": "Smith"}]}))
            .unwrap();
    let outcome = validate_profile_by_url(FHIRProfileArguments::new(resolver), A, &patient)
        .await
        .unwrap();
    let diagnostics = diagnostics(&outcome.issue);
    assert_eq!(diagnostics.len(), 2, "{diagnostics:?}");
    assert!(diagnostics.iter().any(|d| d.contains("Patient.identifier")));
    assert!(diagnostics.iter().any(|d| d.contains("Patient.gender")));
}

/// A discriminator path that no element answers to is reported as such, also
/// when the search can follow profile links in a cycle at every step.
#[tokio::test]
async fn an_unresolvable_discriminator_path_through_a_profile_cycle_is_an_error() {
    const URL: &str = "http://example.org/StructureDefinition/sliced-patient";
    let resolver = Fixed::new([structure_definition(
        URL,
        "Patient",
        &json!([
            {"id": "Patient", "path": "Patient", "min": 0, "max": "*",
             "type": [{"code": "Patient", "profile": [URL, URL, URL, URL, URL, URL, URL, URL]}]},
            {"id": "Patient.identifier", "path": "Patient.identifier", "min": 0, "max": "*",
             "type": [{"code": "Identifier"}],
             "slicing": {"discriminator": [{"type": "value", "path": "nowhere.at.all"}], "rules": "open"}},
            {"id": "Patient.identifier:x", "path": "Patient.identifier", "sliceName": "x",
             "min": 0, "max": "1", "type": [{"code": "Identifier"}]}
        ]),
    )]);
    let patient: Resource =
        serde_json::from_value(json!({"resourceType": "Patient", "identifier": [{"value": "1"}]}))
            .unwrap();
    let started = std::time::Instant::now();
    let error = validate_profile_by_url(FHIRProfileArguments::new(resolver), URL, &patient)
        .await
        .expect_err("the discriminator path has no target");
    assert!(
        error
            .to_string()
            .contains("Failed to find element definition"),
        "{error}"
    );
    assert!(started.elapsed().as_secs() < 5, "{:?}", started.elapsed());
}

/// A value set of `codes` that also includes each of `includes` (one include
/// each, so a union).
fn value_set(url: &str, codes: &[&str], includes: &[&str]) -> Value {
    let mut include = vec![json!({
        "system": "http://example.org/codes",
        "concept": codes.iter().map(|code| json!({"code": code})).collect::<Vec<_>>()
    })];
    include.extend(includes.iter().map(|url| json!({"valueSet": [url]})));
    json!({
        "resourceType": "ValueSet",
        "id": url.rsplit('/').next().unwrap(),
        "url": url,
        "status": "active",
        "compose": {"include": include}
    })
}

/// A value set that includes itself, directly or through another, holds the
/// codes of every value set in the cycle and no more; each is read once.
#[tokio::test]
async fn value_sets_that_include_each_other_are_read_once() {
    async fn issues(resolver: &Arc<Fixed>, observation: Resource) -> usize {
        validate_profile_by_url(
            FHIRProfileArguments::new(resolver.clone()),
            URL,
            &observation,
        )
        .await
        .unwrap()
        .issue
        .len()
    }
    const URL: &str = "http://example.org/StructureDefinition/categorised-observation";
    const OUTER: &str = "http://example.org/ValueSet/outer";
    const INNER: &str = "http://example.org/ValueSet/inner";
    let resolver = Fixed::new([
        structure_definition(
            URL,
            "Observation",
            &json!([
                {"id": "Observation", "path": "Observation", "min": 0, "max": "*"},
                {"id": "Observation.category", "path": "Observation.category", "min": 1, "max": "*",
                 "type": [{"code": "CodeableConcept"}],
                 "slicing": {"discriminator": [{"type": "pattern", "path": "$this"}], "rules": "open"}},
                {"id": "Observation.category:bound", "path": "Observation.category", "sliceName": "bound",
                 "min": 1, "max": "1", "type": [{"code": "CodeableConcept"}],
                 "binding": {"strength": "required", "valueSet": OUTER}}
            ]),
        ),
        value_set(OUTER, &["a"], &[INNER, OUTER]),
        value_set(INNER, &["b"], &[OUTER]),
    ]);
    let observation = |code: &str| -> Resource {
        serde_json::from_value(json!({
            "resourceType": "Observation",
            "status": "final",
            "code": {"text": "x"},
            "category": [{"coding": [{"system": "http://example.org/codes", "code": code}]}]
        }))
        .unwrap()
    };
    assert_eq!(
        issues(&resolver, observation("a")).await,
        0,
        "outer's own code fills the slice"
    );
    assert_eq!(
        issues(&resolver, observation("b")).await,
        0,
        "inner's code, included by outer, fills it"
    );
    assert_eq!(
        issues(&resolver, observation("c")).await,
        1,
        "a code in neither leaves the slice empty"
    );
    // The outer value set is resolved once more than it is read: its resource
    // is the cache key. The inner one is reached only by reading.
    let seen = resolver.resolved.lock().unwrap();
    let reads = |url: &str| seen.iter().filter(|seen| *seen == url).count();
    assert!(reads(OUTER) <= 2, "{seen:?}");
    assert_eq!(reads(INNER), 1, "{seen:?}");
}
