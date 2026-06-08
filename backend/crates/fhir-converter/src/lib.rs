use haste_fhir_model::r4::generated::{resources::Patient, types::HumanName};

mod conversions;
mod filters;

pub fn fhir_converter() {
    let template = liquid::ParserBuilder::with_stdlib()
        .filter(filters::fhirpath::FHIRPath)
        .build()
        .unwrap()
        .parse(
            "
        Result: {{ num.value | minus: 3 }} Patient first name 
        {% assign name = patient | fhirpath: 'name[0]' %}
        
            First {{ name[0].given[0] }}
            Last {{ name[0].family }}

            Z {{ name[0].z | default: 'N/A' }}
        ",
        )
        .unwrap();

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
        })
    });

    let output = template.render(&globals).unwrap();

    println!("{}", output);
}
