//! The precision range of a decimal, computed for every indexed number and
//! quantity and for every number, quantity and `ap` query value.

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use haste_fhir_search::indexing_conversion::get_decimal_range;

fn ranges(c: &mut Criterion) {
    let mut group = c.benchmark_group("decimal_range");
    for value in ["45", "45.123", "-0.000123", "123456789.5", "1e2", "1.5E-3"] {
        group.bench_function(value, |b| b.iter(|| get_decimal_range(black_box(value))));
    }
    group.finish();
}

criterion_group!(benches, ranges);
criterion_main!(benches);
