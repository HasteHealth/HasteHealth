pub fn fhir_converter() {
    let template = liquid::ParserBuilder::with_stdlib()
        .build()
        .unwrap()
        .parse("Liquid! {{num | minus: 2}}")
        .unwrap();

    let globals = liquid::object!({
        "num": 4f64
    });

    let output = template.render(&globals).unwrap();
    println!("{}", output);
}
