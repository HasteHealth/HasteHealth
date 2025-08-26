use clap::{Parser, Subcommand, ValueEnum};
use oxidized_codegen::type_gen;
use oxidized_fhirpath::FHIRPathError;
use std::{
    io::Write,
    process::{Command, Stdio},
};
use thiserror::Error;

#[derive(Parser)]
#[command(version, about, long_about = None)] // Read from `Cargo.toml`
struct Cli {
    #[command(subcommand)]
    command: CLICommand,
}

#[derive(Clone, ValueEnum)]
enum GenerateLevel {
    Primitive,
    Complex,
    Resource,
}

#[derive(Subcommand)]
enum CLICommand {
    FHIRPath {
        /// lists test values
        fhirpath: String,
        /// FHIR data to evaluate the FHIRPath on
        data: String,
    },
    Generate {
        /// Input FHIR StructureDefinition file (JSON)
        #[command(subcommand)]
        command: CodeGen,
    },
}

#[derive(Subcommand)]
enum CodeGen {
    Types {
        #[arg(short, long)]
        input: Vec<String>,
        /// Output Rust file path
        #[arg(short, long)]
        output: Option<String>,
        #[arg(short, long)]
        level: Option<GenerateLevel>,
    },
    Operations {
        #[arg(short, long)]
        input: Vec<String>,
        /// Output Rust file path
        #[arg(short, long)]
        output: Option<String>,
    },
}

#[derive(Debug, Error)]
enum CLIError {
    #[error("FHIRPath error: {0}")]
    FHIRPathError(#[from] FHIRPathError),
    #[error("Generation error: {0}")]
    GenerationError(String),
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("JSON parsing error: {0}")]
    JSONError(#[from] serde_json::Error),
}

fn parse_fhir_data(data: &str) -> Result<serde_json::Value, CLIError> {
    let data: serde_json::Value = serde_json::from_str(data)?;
    Ok(data)
}

fn format_code(rust_code: String) -> String {
    let mut format_command = Command::new("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn child process");

    let mut stdin = format_command.stdin.take().expect("Failed to open stdin");
    std::thread::spawn(move || {
        stdin
            .write_all(rust_code.as_bytes())
            .expect("Failed to write to stdin");
    });

    let command_output = format_command
        .wait_with_output()
        .expect("Failed to read stdout");

    let formatted_code = String::from_utf8_lossy(&command_output.stdout);

    formatted_code.to_string()
}

fn main() -> Result<(), CLIError> {
    let cli = Cli::parse();
    match &cli.command {
        CLICommand::FHIRPath { fhirpath, data } => {
            let data = parse_fhir_data(data)?;
            println!("FHIRPath: {} {}", fhirpath, data);

            // let result = engine.evaluate(fhirpath, vec![&data])?;

            // println!("{:#?}", result.iter().collect::<Vec<_>>());

            Ok(())
        }
        CLICommand::Generate { command } => match command {
            CodeGen::Operations { input, output } => {
                let generated_operation_definitions =
                    type_gen::operation_definitions::generate_operation_definitions_from_files(
                        input,
                    )
                    .map_err(|e| CLIError::GenerationError(e))?;

                let formatted_code = format_code(generated_operation_definitions);

                match output {
                    Some(output_path) => {
                        std::fs::write(output_path, formatted_code.to_string())?;
                        println!("Generated FHIR types written to: {}", output_path);
                    }
                    None => {
                        println!("{}", formatted_code);
                    }
                }

                Ok(())
            }
            CodeGen::Types {
                input,
                output,
                level,
            } => {
                let level = {
                    match level {
                        Some(GenerateLevel::Primitive) => Some("primitive-type"),
                        Some(GenerateLevel::Complex) => Some("complex-type"),
                        Some(GenerateLevel::Resource) => Some("resource"),
                        None => None,
                    }
                };

                let rust_code =
                    type_gen::fhir_rust_types::generate_fhir_types_from_files(input, level)
                        .map_err(|e| CLIError::GenerationError(e))?;

                let formatted_code = format_code(rust_code);

                match output {
                    Some(output_path) => {
                        std::fs::write(output_path, formatted_code.to_string())?;
                        println!("Generated FHIR types written to: {}", output_path);
                    }
                    None => {
                        println!("{}", formatted_code);
                    }
                }

                Ok(())
            }
        },
    }
}
