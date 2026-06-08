mod conversions;
mod filters;

pub fn fhir_converter() {
    let template = liquid::ParserBuilder::with_stdlib()
        .filter(filters::fhirpath::FHIRPath)
        .build()
        .unwrap()
        .parse("Result: {{num | fhirpath: '$this.value + 3'}}")
        .unwrap();

    let globals = liquid::object!({
        "num": liquid::object!({
            "value": 5
        })
    });

    let output = template.render(&globals).unwrap();

    println!("{}", output);
}
