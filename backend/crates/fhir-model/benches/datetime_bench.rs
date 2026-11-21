//! Parsing of the FHIR date and time primitives, which every resource with
//! a date goes through.

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use haste_fhir_model::r4::datetime::{
    Date, DateTime, parse_date, parse_datetime, parse_instant, parse_time,
};

fn parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("datetime_parse");
    group.bench_function("date year", |b| b.iter(|| parse_date(black_box("2024"))));
    group.bench_function("date month", |b| {
        b.iter(|| parse_date(black_box("2024-03")))
    });
    group.bench_function("date day", |b| {
        b.iter(|| parse_date(black_box("2024-03-15")))
    });
    group.bench_function("dateTime day", |b| {
        b.iter(|| parse_datetime(black_box("2024-03-15")))
    });
    group.bench_function("dateTime full", |b| {
        b.iter(|| parse_datetime(black_box("2024-03-15T10:30:00.123+02:00")))
    });
    group.bench_function("instant", |b| {
        b.iter(|| parse_instant(black_box("2024-03-15T10:30:00.123Z")))
    });
    group.bench_function("time", |b| b.iter(|| parse_time(black_box("10:30:00.123"))));
    group.finish();
}

fn rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("datetime_render");
    let year = Date::Year(2024);
    let day = DateTime::YearMonthDay(2024, 3, 15);
    group.bench_function("date year", |b| b.iter(|| black_box(&year).to_string()));
    group.bench_function("dateTime day", |b| b.iter(|| black_box(&day).to_string()));
    group.finish();
}

criterion_group!(benches, parsing, rendering);
criterion_main!(benches);
