use clap::{Parser, Subcommand, ValueEnum};
use oxidized_codegen::type_gen;
use oxidized_fhir_operation_error::{OperationOutcomeCodes, OperationOutcomeError};
use quote::quote;
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

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
        output: String,
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

fn parse_fhir_data(data: &str) -> Result<serde_json::Value, OperationOutcomeError> {
    let data: serde_json::Value = serde_json::from_str(data).map_err(|e| {
        OperationOutcomeError::error(OperationOutcomeCodes::Exception, e.to_string())
    })?;
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

#[tokio::main]
async fn main() -> Result<(), OperationOutcomeError> {
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
                    .map_err(|e| {
                        OperationOutcomeError::error(OperationOutcomeCodes::Exception, e)
                    })?;

                let formatted_code = format_code(generated_operation_definitions);

                match output {
                    Some(output_path) => {
                        std::fs::write(output_path, formatted_code.to_string()).map_err(|e| {
                            OperationOutcomeError::error(
                                OperationOutcomeCodes::Exception,
                                e.to_string(),
                            )
                        })?;
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

                let rust_code = type_gen::rust_types::generate(input, level).await?;
                let output_path = Path::new(output);
                let resource_path = output_path.join("resources.rs");
                std::fs::write(resource_path, format_code(rust_code.resources.to_string()))
                    .map_err(|e| {
                        OperationOutcomeError::error(
                            OperationOutcomeCodes::Exception,
                            e.to_string(),
                        )
                    })?;

                let type_path = output_path.join("types.rs");
                std::fs::write(type_path, format_code(rust_code.types.to_string())).map_err(
                    |e| {
                        OperationOutcomeError::error(
                            OperationOutcomeCodes::Exception,
                            e.to_string(),
                        )
                    },
                )?;

                let terminology_path = output_path.join("terminology.rs");
                std::fs::write(
                    terminology_path,
                    format_code(rust_code.terminology.to_string()),
                )
                .map_err(|e| {
                    OperationOutcomeError::error(OperationOutcomeCodes::Exception, e.to_string())
                })?;

                let mod_path = output_path.join("mod.rs");
                let module_code = quote! {
                    /// DO NOT EDIT THIS FILE. It is auto-generated by the FHIR Rust code generator.
                   pub mod resources;
                   pub mod types;
                   pub mod terminology;
                };
                std::fs::write(mod_path, module_code.to_string()).map_err(|e| {
                    OperationOutcomeError::error(OperationOutcomeCodes::Exception, e.to_string())
                })?;

                let mod_path = output_path.join("mod.rs");
                let module_code = quote! {
                    /// DO NOT EDIT THIS FILE. It is auto-generated by the FHIR Rust code generator.
                   pub mod resources;
                   pub mod types;
                   pub mod terminology;
                };
                std::fs::write(mod_path, format_code(module_code.to_string())).map_err(|e| {
                    OperationOutcomeError::error(OperationOutcomeCodes::Exception, e.to_string())
                })?;

                println!("Generated FHIR types written to: {}", output_path.display());
                Ok(())
            }
        },
    }
}
