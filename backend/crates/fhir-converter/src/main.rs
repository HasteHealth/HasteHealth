use std::collections::HashMap;

use haste_fhir_converter::{create_environment, transform};
use haste_hl7v2::parser::ParsedHL7V2Message;
use minijinja::Value;

#[tokio::main]
async fn main() {
    let mut env = create_environment();

    env.add_template(
        "hello",
        "{{ 'Na '|repeat(3) | hl7v2_segments }} {{ name }} {{ hl7v2.PID[0][4][0] }}!",
    )
    .unwrap();
    let tmpl = env.get_template("hello").unwrap();

    let hl7v2 = ParsedHL7V2Message::try_from(HL7V2_MESSAGE)
        .expect("Failed to parse HL7V2 message")
        .0;

    let mut ctx = HashMap::<&str, Value>::new();

    ctx.insert("name", "Haste".to_string().into());
    ctx.insert(
        "hl7v2",
        Value::from_dyn_object(Arc::new(JHL7V2::new(hl7v2))),
    );
    transform(tmpl, ctx, OutputFormat::FHIR).expect("Failed to transform template");
}
