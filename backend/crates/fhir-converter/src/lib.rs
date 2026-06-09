use haste_fhir_model::r4::generated::{resources::Patient, types::HumanName};
use haste_hl7v2::parser::ParsedHL7V2Message;

mod conversions;
mod filters;

static HL7V2_MESSAGE: &str = include_str!("../test_data/message1.bin");

pub fn fhir_converter() {
    let template = liquid::ParserBuilder::with_stdlib()
        .filter(filters::fhirpath::FHIRPath)
        .filter(filters::hl7v2::HL7V2)
        .build()
        .unwrap()
        .parse(
            "
        Result: {{ num.value | minus: 3 }} Patient first name 
        Result {{hl7v2 | fhirpath: \"$this.segments.where($this.id = 'MSH').fields[1].value.value.value\" | first}}
        {% assign name = patient | fhirpath: \"$this.name\" %}
        
            First {{ name[0].given[0] }}
            Last {{ name[0].family }}

        ",
        )
        .unwrap();

    let hl7v2 = ParsedHL7V2Message::try_from(HL7V2_MESSAGE)
        .expect("Failed to parse HL7V2 message")
        .0;

    // println!("{}", serde_json::to_string_pretty(&hl7v2).unwrap());

    let mut patient = Patient::default();
    patient.name = Some(vec![Box::new(HumanName {
        family: Some(Box::new("Smith".to_string().into())),
        given: Some(vec![Box::new("John".to_string().into())]),
        ..Default::default()
    })]);

    let globals = liquid::object!({
        "patient": liquid::to_object(&patient).unwrap(),
        "num": liquid::object!({
            "value": 5
        }),
        "hl7v2": liquid::to_object(&hl7v2).unwrap(),
    });

    let output = template.render(&globals).unwrap();

    println!("{}", output);
}
