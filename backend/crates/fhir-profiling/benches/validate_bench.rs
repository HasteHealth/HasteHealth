use criterion::{Criterion, criterion_group, criterion_main};
use haste_fhir_client::canonical_resolver::CanonicalResolver;
use haste_fhir_model::r4::generated::resources::{Observation, Resource, ResourceType};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_profiling::{FHIRProfileArguments, validate_profile_by_url};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

const PROFILE_URL: &str = "http://hl7.org/fhir/us/core/StructureDefinition/us-core-vital-signs";

struct Resolver {
    profile: Arc<Resource>,
    calls: AtomicUsize,
}

impl CanonicalResolver for Resolver {
    fn resolve(
        &self,
        _t: ResourceType,
        _u: &str,
    ) -> impl Future<Output = Result<Option<Arc<Resource>>, OperationOutcomeError>> + Send {
        self.calls.fetch_add(1, Ordering::Relaxed);
        std::future::ready(Ok(Some(self.profile.clone())))
    }
}

/// An Observation with `categories` category entries, where only the first
/// matches the profile's `VSCat` slice.
fn observation(categories: usize) -> Observation {
    let category: Vec<_> = (0..categories)
        .map(|i| {
            let code = if i == 0 {
                "vital-signs".to_string()
            } else {
                format!("other-{i}")
            };
            serde_json::json!({"coding": [{"system": "http://terminology.hl7.org/CodeSystem/observation-category", "code": code}]})
        })
        .collect();
    serde_json::from_value(serde_json::json!({
        "resourceType": "Observation",
        "status": "final",
        "category": category,
        "code": {"coding": [{"system": "http://loinc.org", "code": "85354-9"}]},
        "subject": {"reference": "Patient/1"},
        "effectiveDateTime": "2024-01-01"
    }))
    .unwrap()
}

fn bench(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../artifacts/r4/hl7.fhir.us.core/StructureDefinition-us-core-vital-signs.json"
    );
    let sd: Resource = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let resolver = Arc::new(Resolver {
        profile: Arc::new(sd),
        calls: AtomicUsize::new(0),
    });

    for n in [1usize, 50, 200] {
        let obs = observation(n);
        let before = resolver.calls.load(Ordering::Relaxed);
        let out = rt
            .block_on(validate_profile_by_url(
                FHIRProfileArguments::new(resolver.clone()),
                PROFILE_URL,
                &obs,
            ))
            .unwrap();
        eprintln!(
            "categories={n} issues={} resolver_calls={}",
            out.issue.len(),
            resolver.calls.load(Ordering::Relaxed) - before
        );
        c.bench_function(&format!("validate_vital_signs_{n}_categories"), |b| {
            b.iter(|| {
                rt.block_on(validate_profile_by_url(
                    FHIRProfileArguments::new(resolver.clone()),
                    PROFILE_URL,
                    &obs,
                ))
                .unwrap()
            });
        });
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);
