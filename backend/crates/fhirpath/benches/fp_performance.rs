use criterion::{Criterion, criterion_group, criterion_main};
use haste_fhir_model::r4::generated::{
    resources::Patient,
    types::{FHIRString, HumanName},
};
use haste_fhirpath::FPEngine;
use tokio::runtime::Runtime;

fn fp_performance_simple(c: &mut Criterion) {
    let root = Patient {
        name: Some(vec![Box::new(HumanName {
            given: Some(vec![Box::new(FHIRString {
                value: Some("John".to_string()),
                ..Default::default()
            })]),
            ..Default::default()
        })]),
        ..Default::default()
    };
    let engine = FPEngine::new();
    c.bench_function("fp_performance_simple", |b| {
        b.to_async(Runtime::new().unwrap())
            .iter(|| engine.evaluate("Patient.name.given", vec![&root]))
    });
}

fn parser_test_performance(c: &mut Criterion) {
    let engine = FPEngine::new();
    c.bench_function("parser_test_performance", |b| {
        b.to_async(Runtime::new().unwrap())
            .iter(|| engine.evaluate("1 + 2 * (3 - 4) / 5", vec![]))
    });
}

fn parser_test_complex(c: &mut Criterion) {
    let engine = FPEngine::new();
    c.bench_function("parser_test_complex",
    |b|  b.to_async(Runtime::new().unwrap()).iter(|| engine.evaluate("$this.field + %test._asdf.test(45, $this.field) * 64 * $this.where($this.field = '23'.length())", vec![])));
}

fn parser_test_simple(c: &mut Criterion) {
    let engine = FPEngine::new();
    c.bench_function("parser_test_simple", |b| {
        b.to_async(Runtime::new().unwrap())
            .iter(|| engine.evaluate("$this.field", vec![]))
    });
}

static SYNTHEA_PATIENT: &str = include_str!("../data/synthea_patient.json");

static PATIENT_PARAMETER_EXPRESSIONS: &[&str] = &[
    "Patient.active",
    "Patient.address",
    "Patient.address.city",
    "Patient.address.country",
    "Patient.address.postalCode",
    "Patient.address.state",
    "Patient.address.use",
    "Patient.birthDate",
    "(Patient.deceased as dateTime)",
    "Patient.deceased.exists() and Patient.deceased != false",
    "Patient.telecom.where(system='email')",
    "Patient.name.family",
    "Patient.gender",
    "Patient.generalPractitioner",
    "Patient.name.given",
    "Patient.identifier",
    "Patient.communication.language",
    "Patient.link.other",
    "Patient.name",
    "Patient.managingOrganization",
    "Patient.telecom.where(system='phone')",
    "Patient.name",
    "Patient.telecom",
];

async fn index_patient(
    engine: &FPEngine,
    p: &Patient,
) -> Result<(), haste_fhirpath::FHIRPathError> {
    for expr in PATIENT_PARAMETER_EXPRESSIONS {
        let _result = engine.evaluate(expr, vec![p]).await?;
    }

    Ok(())
}

fn index_synthea_patient(c: &mut Criterion) {
    let patient = haste_fhir_serialization_json::from_str::<
        haste_fhir_model::r4::generated::resources::Patient,
    >(SYNTHEA_PATIENT)
    .unwrap();

    let engine = FPEngine::new();

    c.bench_function("index_synthea_patient", |b| {
        b.to_async(Runtime::new().unwrap())
            .iter(|| index_patient(&engine, &patient))
    });
}

criterion_group!(evaluation_tests, index_synthea_patient);

criterion_group!(
    parser_tests,
    fp_performance_simple,
    parser_test_performance,
    parser_test_complex,
    parser_test_simple
);

criterion_main!(evaluation_tests, parser_tests);
