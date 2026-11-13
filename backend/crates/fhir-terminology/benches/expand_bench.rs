use criterion::{Criterion, criterion_group, criterion_main};
use haste_fhir_client::canonical_resolver::CanonicalResolver;
use haste_fhir_generated_ops::generated::ValueSetExpand;
use haste_fhir_model::r4::generated::{
    resources::{
        CodeSystem, CodeSystemConcept, CodeSystemConceptProperty,
        CodeSystemConceptPropertyValueTypeChoice, Resource, ResourceType, ValueSet,
        ValueSetCompose, ValueSetComposeInclude,
    },
    terminology::CodesystemContentMode,
    types::{FHIRCode, FHIRString, FHIRUri},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_terminology::{FHIRTerminology, client::FHIRCanonicalTerminology};
use std::sync::Arc;

#[derive(Clone)]
struct R(Arc<Resource>, Arc<Resource>);

impl CanonicalResolver for R {
    async fn resolve(
        &self,
        t: ResourceType,
        _u: &str,
    ) -> Result<Option<Arc<Resource>>, OperationOutcomeError> {
        Ok(Some(if t == ResourceType::CodeSystem {
            self.0.clone()
        } else {
            self.1.clone()
        }))
    }
}

fn uri(s: &str) -> Box<FHIRUri> {
    Box::new(FHIRUri {
        value: Some(s.into()),
        ..Default::default()
    })
}

fn string(s: String) -> Box<FHIRString> {
    Box::new(FHIRString {
        value: Some(s),
        ..Default::default()
    })
}

fn value_set_for(system: &str) -> Arc<Resource> {
    Arc::new(Resource::ValueSet(ValueSet {
        url: Some(uri("http://x/vs")),
        compose: Some(ValueSetCompose {
            include: vec![ValueSetComposeInclude {
                system: Some(uri(system)),
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }))
}

fn synthetic(n: usize) -> R {
    let concepts = (0..n)
        .map(|i| CodeSystemConcept {
            code: Box::new(FHIRCode {
                value: Some(format!("code-{i}")),
                ..Default::default()
            }),
            display: Some(string(format!("Display {i}"))),
            definition: Some(string(format!(
                "The formal definition of concept number {i}, as code systems usually carry one."
            ))),
            property: Some(vec![CodeSystemConceptProperty {
                code: Box::new(FHIRCode {
                    value: Some("status".into()),
                    ..Default::default()
                }),
                value: CodeSystemConceptPropertyValueTypeChoice::Code(Box::new(FHIRCode {
                    value: Some("active".into()),
                    ..Default::default()
                })),
                ..Default::default()
            }]),
            ..Default::default()
        })
        .collect();
    let cs = CodeSystem {
        url: Some(uri("http://x/cs")),
        version: Some(string("1.0.0".into())),
        content: CodesystemContentMode::complete(),
        concept: Some(concepts),
        ..Default::default()
    };
    R(
        Arc::new(Resource::CodeSystem(cs)),
        value_set_for("http://x/cs"),
    )
}

fn count(concepts: &[CodeSystemConcept]) -> usize {
    concepts
        .iter()
        .map(|c| 1 + count(c.concept.as_deref().unwrap_or_default()))
        .sum()
}

fn real(path: &str) -> (String, R) {
    let cs: CodeSystem = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let url = cs.url.as_ref().and_then(|u| u.value.clone()).unwrap();
    let name = format!(
        "{}_{}",
        url.rsplit('/').next().unwrap(),
        count(cs.concept.as_deref().unwrap_or_default())
    );
    (
        name,
        R(Arc::new(Resource::CodeSystem(cs)), value_set_for(&url)),
    )
}

fn input() -> ValueSetExpand::Input {
    ValueSetExpand::Input {
        url: Some(FHIRUri {
            value: Some("http://x/vs".into()),
            ..Default::default()
        }),
        valueSet: None,
        valueSetVersion: None,
        context: None,
        contextDirection: None,
        filter: None,
        date: None,
        offset: None,
        count: None,
        includeDesignations: None,
        designation: None,
        includeDefinition: None,
        activeOnly: None,
        excludeNested: None,
        excludeNotForUI: None,
        excludePostCoordinated: None,
        displayLanguage: None,
        exclude_system: None,
        system_version: None,
        check_system_version: None,
        force_system_version: None,
    }
}

fn bench(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let t = FHIRCanonicalTerminology::new();
    let mut cases: Vec<(String, R)> = [1000usize, 10000]
        .into_iter()
        .map(|n| (format!("synthetic_{n}"), synthetic(n)))
        .collect();
    if let Ok(paths) = std::env::var("HASTE_TERMINOLOGY_CODESYSTEMS") {
        cases.extend(paths.split(':').filter(|p| !p.is_empty()).map(real));
    }
    for (name, r) in &cases {
        let out = rt.block_on(t.expand(r.clone(), input())).unwrap();
        let contains = out
            .return_
            .expansion
            .as_ref()
            .and_then(|e| e.contains.as_ref())
            .map_or(0, Vec::len);
        eprintln!("{name}: top-level contains={contains}");
        c.bench_function(&format!("expand_{name}"), |b| {
            b.iter(|| rt.block_on(t.expand(r.clone(), input())).unwrap());
        });
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);
