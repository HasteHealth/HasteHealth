use std::{path::Path, sync::Arc};

use haste_fhir_model::r4::generated::{resources::Resource, terminology::IssueType};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_hl7v2::parser::ParsedHL7V2Message;
use minijinja::{Environment, Template, Value};

mod jinja_extensions;
mod liquid_extensions;

pub enum Input {
    HL7V2(String),
    FHIR(Resource),
    JSON(serde_json::Value),
}

pub fn convert_input(input: Input) -> Result<minijinja::Value, OperationOutcomeError> {
    match input {
        Input::HL7V2(message) => {
            let parsed_message = ParsedHL7V2Message::try_from(message.as_str())?.0;

            Ok(Value::from_dyn_object(Arc::new(
                jinja_extensions::conversions::hl7v2::JHL7V2::new(parsed_message),
            )))
        }
        Input::FHIR(resource) => Ok(minijinja::Value::from_serialize(resource)),
        Input::JSON(json) => Ok(minijinja::Value::from_serialize(json)),
    }
}

#[derive(Clone, Copy, Debug)]
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

// Uses relative path from template directory and strips ending jinja prefix.
fn derive_template_name(template_dir: &Path, path: &Path) -> Option<String> {
    let relative_path = path.strip_prefix(template_dir).unwrap_or(path);
    let Some(template_file_stem) = relative_path.file_stem().and_then(|s| s.to_str()) else {
        eprintln!("Failed to get template name from path: {:?}", path);
        return None;
    };
    let Some(parent) = relative_path.parent() else {
        eprintln!(
            "Failed to get parent directory for template: {:?}",
            relative_path
        );
        return None;
    };

    let template_name_path = parent.join(template_file_stem);

    let Some(template_name) = template_name_path.to_str() else {
        eprintln!(
            "Failed to convert template name to string: {:?}",
            template_name_path
        );
        return None;
    };

    Some(template_name.to_string())
}

fn add_template(env: &mut Environment<'_>, template_dir: &Path, path: &Path) -> Option<()> {
    let Ok(template_content) = std::fs::read_to_string(path) else {
        eprintln!("Failed to read template file: {:?}", path);
        return None;
    };

    let Some(template_name) = derive_template_name(template_dir, path) else {
        eprintln!("Failed to derive template name for file: {:?}", path);
        return None;
    };

    println!("Adding template '{}' from file: {:?}", template_name, path);

    if let Err(e) = env.add_template_owned(template_name.to_string(), template_content) {
        eprintln!("Failed to add template '{}': {}", template_name, e);
    }

    Some(())
}

pub fn create_environment<'a>(template_dir: Option<&str>) -> Environment<'a> {
    let mut env = Environment::new();
    env.add_filter(
        "hl7v2_segments",
        jinja_extensions::filters::hl7v2::hl7v2_segments,
    );

    if let Some(template_dir) = template_dir {
        let template_dir = Path::new(template_dir);
        walkdir::WalkDir::new(template_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_type().is_file()
                    && e.path()
                        .extension()
                        .map_or(false, |ext| ext == "jinja" || ext == "j2")
            })
            .for_each(|entry| {
                let path = entry.path();
                add_template(&mut env, template_dir, path);
            });
    }

    env
}

pub fn transform<S>(
    template: &Template<'_, '_>,
    ctx: S,
    output: &OutputFormat,
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
