use std::{collections::HashMap, sync::Arc};

use haste_hl7v2::parser::ParsedHL7V2Message;
use minijinja::{Environment, Value};

use crate::jinja_extensions::conversions::hl7v2::JHL7V2;

mod jinja_extensions;
mod liquid_extensions;

static HL7V2_MESSAGE: &str = include_str!("../test_data/message1.bin");

pub fn fhir_converter2() {
    let mut env = Environment::new();
    env.add_filter("repeat", str::repeat);
    env.add_filter(
        "hl7v2_segments",
        jinja_extensions::filters::hl7v2::hl7v2_segments,
    );

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

    println!("{}", tmpl.render(ctx).unwrap());
}
