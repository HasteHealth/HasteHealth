use crate::CLIState;
use clap::Subcommand;
use haste_fhir_model::r4::generated::resources::{Resource, TestScript};
use haste_fhir_operation_error::OperationOutcomeError;
use std::{path::Path, sync::Arc};
use tokio::sync::Mutex;
use tracing::info;

#[derive(Subcommand)]
pub enum TestScriptCommands {
    Run {
        #[arg(short, long)]
        input: Vec<String>,
    },
}

fn load_testscript_files(path: &Path) -> Vec<TestScript> {
    let mut testscripts = vec![];

    let Ok(data) = std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))
    else {
        return vec![];
    };

    let resource = match haste_fhir_serialization_json::from_str::<Resource>(&data) {
        Ok(resource) => resource,
        Err(e) => {
            println!(
                "Failed to parse FHIR resource from file {}: {}",
                path.display(),
                e
            );
            return vec![];
        }
    };

    match resource {
        Resource::Bundle(bundle) => bundle
            .entry
            .unwrap_or(vec![])
            .into_iter()
            .for_each(|entry| {
                if let Some(resource) = entry.resource {
                    match *resource {
                        Resource::TestScript(testscript) => {
                            testscripts.push(testscript);
                        }
                        _ => {}
                    }
                }
            }),
        Resource::TestScript(testscript) => {
            testscripts.push(testscript);
        }
        _ => {}
    }

    testscripts
}

pub async fn testscript_commands(
    state: Arc<Mutex<CLIState>>,
    command: &TestScriptCommands,
) -> Result<(), OperationOutcomeError> {
    match command {
        TestScriptCommands::Run { input: inputs } => {
            let fhir_client = crate::client::fhir_client(state).await?;

            for input in inputs {
                let walker = walkdir::WalkDir::new(&input).into_iter();

                for entry in walker
                    .filter_map(|e| e.ok())
                    .filter(|e| e.metadata().unwrap().is_file())
                    .filter(|f| f.file_name().to_string_lossy().ends_with(".json"))
                {
                    let testscripts = load_testscript_files(&entry.path());
                    for testscript in testscripts.into_iter() {
                        info!(
                            "Running TestScript '{}' from file: {}",
                            testscript
                                .name
                                .value
                                .clone()
                                .unwrap_or("<Unnamed TestScript>".to_string()),
                            entry.path().to_string_lossy()
                        );

                        match haste_testscript_runner::run(
                            fhir_client.as_ref(),
                            (),
                            Arc::new(testscript),
                        )
                        .await
                        {
                            Ok(result) => {
                                info!("{:#?}", result);
                            }
                            Err(e) => {
                                info!("Error running TestScript '{:?}'", e);
                            }
                        }
                    }
                }
            }

            Ok(())
        }
    }
}
