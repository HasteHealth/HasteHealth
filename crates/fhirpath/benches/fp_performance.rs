use criterion::{Criterion, criterion_group, criterion_main};
use oxidized_fhir_model::r4::types::{FHIRString, HumanName, Patient};
use oxidized_fhirpath::FPEngine;

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
        b.iter(|| engine.evaluate("Patient.name.given", vec![&root]).unwrap())
    });
}

fn parser_test_performance(c: &mut Criterion) {
    let engine = FPEngine::new();
    c.bench_function("parser_test_performance", |b| {
        b.iter(|| engine.evaluate("1 + 2 * (3 - 4) / 5", vec![]).unwrap())
    });
}

fn parser_test_complex(c: &mut Criterion) {
    let engine = FPEngine::new();
    c.bench_function("parser_test_complex",
    |b| b.iter(|| engine.evaluate("$this.field + %test._asdf.test(45, $this.field) * 64 * $this.where($this.field = '23'.length())", vec![]).unwrap()));
}

fn parser_test_simple(c: &mut Criterion) {
    let engine = FPEngine::new();
    c.bench_function("parser_test_simple", |b| {
        b.iter(|| engine.evaluate("$this.field", vec![]).unwrap())
    });
}

criterion_group!(
    benches,
    fp_performance_simple,
    parser_test_performance,
    parser_test_complex,
    parser_test_simple
);
criterion_main!(benches);
