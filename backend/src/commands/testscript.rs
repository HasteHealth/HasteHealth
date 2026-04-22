use crate::CLIState;
use clap::Subcommand;
use haste_fhir_model::r4::generated::{
    resources::{Bundle, BundleEntry, BundleEntryRequest, Resource, TestScript},
    terminology::{BundleType, HttpVerb, IssueType, ReportResultCodes},
    types::FHIRUri,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_testscript_runner::TestRunnerOptions;
use std::{path::Path, sync::Arc};
use tokio::{sync::Mutex, task::JoinSet};
use tracing::info;

#[derive(Subcommand)]
pub enum TestScriptCommands {
    Run {
        #[arg(short, long)]
        input: Vec<String>,
        #[arg(short, long)]
        output: Option<String>,
        #[arg(short, long)]
        wait_between_operations_ms: Option<u64>,
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
        TestScriptCommands::Run {
            output,
            input: inputs,
            wait_between_operations_ms,
        } => {
            let fhir_client = crate::client::fhir_client(state).await?;

            let mut testreport_entries = vec![];
            let testrunner_options = Arc::new(TestRunnerOptions {
                wait_between_operations: wait_between_operations_ms
                    .map(|ms| std::time::Duration::from_millis(ms)),
            });

            let mut status_code = 0;
            let mut test_runs = JoinSet::new();

            for input in inputs {
                let walker = walkdir::WalkDir::new(&input).into_iter();

                for entry in walker
                    .filter_map(|e| e.ok())
                    .filter(|e| e.metadata().unwrap().is_file())
                    .filter(|f| f.file_name().to_string_lossy().ends_with(".json"))
                {
                    let testscripts = load_testscript_files(&entry.path());
                    for testscript in testscripts.into_iter() {
                        let testscript = Arc::new(testscript);

                        let Some(testscript_id) = testscript.id.as_ref() else {
                            info!(
                                "Skipping TestScript without ID from file: {}",
                                entry.path().to_string_lossy()
                            );
                            continue;
                        };

                        info!(
                            "Running TestScript '{}' from file: {}",
                            testscript
                                .name
                                .value
                                .clone()
                                .unwrap_or("<Unnamed TestScript>".to_string()),
                            entry.path().to_string_lossy()
                        );

                        let testscript_id = testscript_id.clone();
                        let testrunner_options = testrunner_options.clone();
                        let fhir_client = fhir_client.clone();

                        test_runs.spawn(async move {
                            match haste_testscript_runner::run(
                                fhir_client.as_ref(),
                                (),
                                testscript,
                                testrunner_options,
                            )
                            .await
                            {
                                Ok(mut test_report) => {
                                    test_report.id = Some(testscript_id);
                                    Ok(test_report)
                                }
                                Err(e) => Err(e),
                            }
                        });
                    }
                }
            }

            while let Some(Ok(res)) = test_runs.join_next().await {
                match res {
                    Ok(test_report) => {
                        match test_report.result.as_ref() {
                            ReportResultCodes::Fail(_) => status_code = 1,
                            // Ignore for rest.
                            ReportResultCodes::Pass(_)
                            | ReportResultCodes::Pending(_)
                            | ReportResultCodes::Null(_) => {}
                        }

                        testreport_entries.push(BundleEntry {
                            request: Some(BundleEntryRequest {
                                method: Box::new(HttpVerb::PUT(None)),
                                url: Box::new(FHIRUri {
                                    value: Some(format!(
                                        "TestReport/{}",
                                        test_report.id.as_ref().map(|id| id.as_str()).unwrap_or("")
                                    )),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            }),
                            resource: Some(Box::new(Resource::TestReport(test_report))),
                            ..Default::default()
                        });
                    }
                    Err(e) => {
                        info!("Error running TestScript '{:?}'", e);
                    }
                }
            }

            let testreport_bundle = Bundle {
                type_: Box::new(BundleType::Transaction(None)),
                entry: Some(testreport_entries),
                ..Default::default()
            };

            if let Some(output) = output {
                std::fs::write(
                    output,
                    haste_fhir_serialization_json::to_string(&testreport_bundle).map_err(|e| {
                        OperationOutcomeError::fatal(
                            IssueType::Exception(None),
                            format!("Failed to serialize TestReport bundle: {}", e),
                        )
                    })?,
                )
                .expect("Failed to write TestReport bundle to file");
            } else {
                println!(
                    "{}",
                    haste_fhir_serialization_json::to_string(&testreport_bundle).map_err(|e| {
                        OperationOutcomeError::fatal(
                            IssueType::Exception(None),
                            format!("Failed to serialize TestReport bundle: {}", e),
                        )
                    })?
                );
            }

            if status_code != 0 {
                Err(OperationOutcomeError::fatal(
                    IssueType::Exception(None),
                    "One or more TestScripts failed".to_string(),
                ))
            } else {
                Ok(())
            }
        }
    }
}
