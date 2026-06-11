use std::{collections::HashMap, io::Read};

use clap::{Arg, Command};
use haste_fhir_converter::{
    Input, Output, OutputFormat, convert_input, create_environment, transform,
};
use haste_fhir_model::r4::generated::resources::Resource;
use minijinja::Value;

#[derive(Clone, Copy, Debug)]
enum InputType {
    HL7V2,
    Fhir,
    Json,
}

fn parse_input_type(value: &str) -> Result<InputType, String> {
    match value.to_ascii_lowercase().as_str() {
        "hl7v2" => Ok(InputType::HL7V2),
        "fhir" => Ok(InputType::Fhir),
        "json" => Ok(InputType::Json),
        _ => Err("invalid input type; expected one of: hl7v2, fhir, json".to_string()),
    }
}

fn parse_output_type(value: &str) -> Result<OutputFormat, String> {
    match value.to_ascii_lowercase().as_str() {
        "hl7v2" => Ok(OutputFormat::HL7V2),
        "fhir" => Ok(OutputFormat::FHIR),
        "json" => Ok(OutputFormat::JSON),
        _ => Err("invalid output type; expected one of: hl7v2, fhir, json".to_string()),
    }
}

fn read_stdin() -> Result<String, std::io::Error> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;
    Ok(input)
}

fn parse_input_value(input_type: &InputType, raw_input: String) -> Result<Input, String> {
    match input_type {
        InputType::HL7V2 => Ok(Input::HL7V2(raw_input)),
        InputType::Fhir => serde_json::from_str::<Resource>(&raw_input)
            .map(Input::FHIR)
            .map_err(|e| format!("failed to parse FHIR input: {e}")),
        InputType::Json => serde_json::from_str::<serde_json::Value>(&raw_input)
            .map(Input::JSON)
            .map_err(|e| format!("failed to parse JSON input: {e}")),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let command = Command::new("haste-fhir-converter")
        .about("Transforms template input from one format to another")
        .arg(
            Arg::new("template")
                .long("template")
                .short('t')
                .required(true)
                .value_name("TEMPLATE")
                .help("Template string used for transformation"),
        )
        .arg(
            Arg::new("input-type")
                .long("input-type")
                .required(true)
                .value_name("TYPE")
                .value_parser(parse_input_type)
                .help("Input type: hl7v2, fhir, json"),
        )
        .arg(
            Arg::new("output-type")
                .long("output-type")
                .required(true)
                .value_name("TYPE")
                .value_parser(parse_output_type)
                .help("Output type: hl7v2, fhir, json"),
        )
        .arg(
            Arg::new("input")
                .long("input")
                .short('i')
                .value_name("INPUT")
                .help("Raw input content. If omitted, stdin is used"),
        )
        .get_matches();

    let template_source = command
        .get_one::<String>("template")
        .expect("template is required");

    let input_type = command
        .get_one::<InputType>("input-type")
        .expect("input-type is required");

    let output_type = command
        .get_one::<OutputFormat>("output-type")
        .expect("output-type is required");

    let raw_input = match command.get_one::<String>("input") {
        Some(input) => input.clone(),
        None => read_stdin()?,
    };

    let input = parse_input_value(input_type, raw_input)?;
    let converted_input = convert_input(input).map_err(|e| e.to_string())?;

    let mut context = HashMap::<&str, Value>::new();
    context.insert("input", converted_input.clone());

    match input_type {
        InputType::HL7V2 => {
            context.insert("hl7v2", converted_input);
        }
        InputType::Fhir => {
            context.insert("fhir", converted_input);
        }
        InputType::Json => {
            context.insert("json", converted_input);
        }
    }

    let template = std::fs::read_to_string(template_source)
        .map_err(|e| format!("failed to read template file: {e}"))?;

    let env = create_environment();
    let template = env
        .template_from_str(&template)
        .map_err(|e| e.to_string())?;

    let output = transform(&template, context, output_type).map_err(|e| e.to_string())?;

    match output {
        Output::FHIR(resource) => {
            println!("{}", serde_json::to_string_pretty(&resource)?);
        }
        Output::JSON(json) => {
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        Output::HL7V2(message) => {
            println!("{message}");
        }
    }

    Ok(())
}
