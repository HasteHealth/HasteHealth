use std::{collections::HashMap, sync::Arc};

use haste_fhir_model::r4::generated::{resources::Resource, terminology::IssueType};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_hl7v2::parser::ParsedHL7V2Message;
use minijinja::{Environment, Template, Value};

use crate::jinja_extensions::conversions::hl7v2::JHL7V2;

mod jinja_extensions;
mod liquid_extensions;

static HL7V2_MESSAGE: &str = include_str!("../test_data/message1.bin");

pub enum OutputFormat {
    FHIR,
    JSON,
    HL7V2,
}

pub enum Output {
    FHIR(Resource),
    JSON(serde_json::Value),
    HL7V2(String),
}

pub fn create_environment<'a>() -> Environment<'a> {
    let mut env = Environment::new();
    env.add_filter(
        "hl7v2_segments",
        jinja_extensions::filters::hl7v2::hl7v2_segments,
    );

    env
}

pub fn transform<S>(
    template: &Template<'_, '_>,
    ctx: S,
    output: OutputFormat,
) -> Result<Output, OperationOutcomeError>
where
    S: serde::Serialize,
{
    let output_data = template
        .render(ctx)
        .map_err(|e| OperationOutcomeError::error(IssueType::Invalid(None), e.to_string()))?;

    match output {
        OutputFormat::FHIR => Ok(Output::FHIR(serde_json::from_str(&output_data).map_err(
            |e| OperationOutcomeError::error(IssueType::Invalid(None), e.to_string()),
        )?)),
        OutputFormat::JSON => Ok(Output::JSON(
            serde_json::from_str::<serde_json::Value>(&output_data).map_err(|e| {
                OperationOutcomeError::error(IssueType::Invalid(None), e.to_string())
            })?,
        )),
        OutputFormat::HL7V2 => {
            // Verify that the output is a valid HL7v2 message by attempting to parse it.
            ParsedHL7V2Message::try_from(output_data.as_str())?.0;

            Ok(Output::HL7V2(output_data))
        }
    }
}
