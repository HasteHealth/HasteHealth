use criterion::{Criterion, criterion_group, criterion_main};
use haste_hl7v2::{parser::ParsedHL7V2Message, serialize::SerializeMessage};

fn hl7_single_message_parse(c: &mut Criterion) {
    let hl7v2_message = include_str!("../test_data/message1.bin");

    c.bench_function("simple message", |b| {
        b.iter(|| {
            ParsedHL7V2Message::try_from(hl7v2_message).expect("Failed to parse HL7v2 message");
        });
    });

    c.bench_function("simple message round trip", |b| {
        b.iter(|| {
            let message =
                ParsedHL7V2Message::try_from(hl7v2_message).expect("Failed to parse HL7v2 message");
            let _ = std::convert::Into::<SerializeMessage<'_>>::into(SerializeMessage(&message.0));
        });
    });
}

criterion_group!(benches, hl7_single_message_parse);
criterion_main!(benches);
