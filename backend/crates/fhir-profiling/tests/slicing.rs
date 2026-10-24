use haste_fhir_client::canonical_resolver::CanonicalResolver;
use haste_fhir_model::r4::generated::resources::{Resource, ResourceType};
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
