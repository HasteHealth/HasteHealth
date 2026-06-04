use criterion::{Criterion, criterion_group, criterion_main};
use haste_hl7v2::parser::ParsedHL7V2Message;

fn hl7_single_message(c: &mut Criterion) {
    let hl7v2_message = include_str!("../test_data/message1.bin");

    c.bench_function("simple message", |b| {
        b.iter(|| {
            let _message = ParsedHL7V2Message::try_from(hl7v2_message);
            _message.expect("Failed to parse HL7v2 message");
        })
    });
}

criterion_group!(benches, hl7_single_message);
criterion_main!(benches);
