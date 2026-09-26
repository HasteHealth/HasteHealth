use crate::cli::state::CliState;
use clap::Subcommand;
use haste_fhir_client::http::{HeaderMap, HttpRequestHeaders, WithRequestHeaders};
use haste_fhir_model::r4::generated::{
    resources::{Bundle, BundleEntry, BundleEntryRequest, Resource, TestReport, TestScript},
    terminology::{BundleType, HttpVerb, IssueType, ReportResultCodes},
    types::{Extension, ExtensionValueTypeChoice, FHIRBoolean, FHIRUri},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_testscript_runner::TestRunnerOptions;
use std::{path::Path, sync::Arc};
use tokio::{sync::Mutex, task::JoinSet};
use tracing::{error, info, warn};

/// Per-operation client context for a TestScript run.
///
/// Carries the headers declared by an operation's `requestHeader` entries so they
/// reach the outgoing HTTP request. Commands that never set headers use `()` instead.
#[derive(Debug, Default, Clone)]
pub(crate) struct TestScriptContext {
    headers: Option<HeaderMap>,
}

impl HttpRequestHeaders for TestScriptContext {
    fn request_headers(&self) -> Option<&HeaderMap> {
        self.headers.as_ref()
    }
}

impl WithRequestHeaders for TestScriptContext {
    fn with_request_headers(mut self, headers: HeaderMap) -> Self {
        self.headers = Some(headers);
        self
    }
}

/// Run FHIR TestScript resources against the active profile's server.
#[derive(Subcommand, Debug)]
pub(crate) enum TestScriptCommands {
    /// Run every TestScript resource found under the given input path(s), in parallel,
    /// and write a transaction Bundle of the resulting TestReports.
    Run {
        /// File or directory to search for TestScript resources (JSON). Repeatable.
        #[arg(short, long)]
        input: Vec<String>,
        /// Write the resulting TestReport bundle to this file instead of stdout.
        #[arg(short, long)]
        output: Option<String>,
        /// Delay between operations within a TestScript, in milliseconds.
        #[arg(short, long)]
        wait_between_operations_ms: Option<u64>,
        /// Before a search (or conditional delete/update), wait up to this many
        /// milliseconds for the TestScript's latest write to be indexed. Search
        /// indexing is asynchronous; this waits only as long as it takes.
        #[arg(long)]
        index_wait_ms: Option<u64>,
        /// A TestScript id whose failure is known and shouldn't fail the run.
        /// Its TestReport is still recorded as `fail`. Repeatable.
        #[arg(long = "allow-failure", value_name = "TESTSCRIPT_ID")]
        allowed_failures: Vec<String>,
    },
}

/// The TestScripts in a file (a TestScript or a Bundle of them). A file that
/// can't be read or parsed is an error, so a broken script fails the run
/// instead of silently not running.
fn load_testscript_files(path: &Path) -> Result<Vec<TestScript>, String> {
    let mut testscripts = vec![];

    let data = std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {e}"))?;
    let resource = serde_json::from_str::<Resource>(&data)
        .map_err(|e| format!("Failed to parse FHIR resource: {e}"))?;

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

    Ok(testscripts)
}

/// Marks a TestReport whose failure is known (`--allow-failure`). It stays
/// `fail`, so the result is truthful, but readers such as the website's
/// coverage page can show its failures as expected rather than as errors.
const EXPECTED_FAILURE_URL: &str =
    "https://haste.health/fhir/StructureDefinition/testreport-expected-failure";

fn mark_expected_failure(test_report: &mut TestReport) {
    test_report
        .extension
        .get_or_insert_with(Vec::new)
        .push(Extension {
            url: EXPECTED_FAILURE_URL.to_string(),
            value: Some(ExtensionValueTypeChoice::Boolean(Box::new(FHIRBoolean {
                value: Some(true),
                ..Default::default()
            }))),
            ..Default::default()
        });
}

/// Runs the `testscript` command group.
pub(crate) async fn run(
    state: Arc<Mutex<CliState>>,
    command: &TestScriptCommands,
) -> Result<(), OperationOutcomeError> {
    match command {
        TestScriptCommands::Run {
            output,
            input: inputs,
            wait_between_operations_ms,
            index_wait_ms,
            allowed_failures,
        } => {
            let fhir_client = crate::cli::client::fhir_client(state).await?;

            let mut testreport_entries = vec![];
            let testrunner_options = Arc::new(TestRunnerOptions {
                wait_between_operations: wait_between_operations_ms
                    .map(|ms| std::time::Duration::from_millis(ms)),
                index_wait_timeout: index_wait_ms.map(std::time::Duration::from_millis),
            });

            let mut status_code = 0;
            let mut test_runs = JoinSet::new();
            let is_allowed_failure = |test_report: &TestReport| {
                test_report
                    .id
                    .as_ref()
                    .is_some_and(|id| allowed_failures.contains(id))
            };

            for input in inputs {
                let walker = walkdir::WalkDir::new(&input).into_iter();

                for entry in walker
                    .filter_map(|e| e.ok())
                    .filter(|e| e.metadata().unwrap().is_file())
                    .filter(|f| f.file_name().to_string_lossy().ends_with(".json"))
                {
                    println!("Processing file: {}", entry.path().display());
                    let testscripts = match load_testscript_files(entry.path()) {
                        Ok(testscripts) => testscripts,
                        Err(e) => {
                            status_code = 1;
                            error!("Skipping {}: {e}", entry.path().display());
                            continue;
                        }
                    };
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
                        let testscript_name = testscript
                            .name
                            .value
                            .clone()
                            .unwrap_or("<Unnamed TestScript>".to_string());
                        let testscript_file = entry.path().to_string_lossy().to_string();
                        let testrunner_options = testrunner_options.clone();
                        let fhir_client = fhir_client.clone();

                        test_runs.spawn(async move {
                            let result = haste_testscript_runner::run(
                                fhir_client.as_ref(),
                                TestScriptContext::default(),
                                testscript,
                                testrunner_options,
                            )
                            .await;

                            (
                                testscript_name,
                                testscript_file,
                                match result {
                                    Ok(mut test_report) => {
                                        test_report.id = Some(testscript_id);
                                        Ok(test_report)
                                    }
                                    Err(e) => Err(e),
                                },
                            )
                        });
                    }
                }
            }

            while let Some(Ok((testscript_name, testscript_file, res))) =
                test_runs.join_next().await
            {
                match res {
                    Ok(mut test_report) => {
                        match &test_report.result.clone() {
                            // Ignore for rest.
                            r if r == &ReportResultCodes::pass()
                                || r == &ReportResultCodes::pending()
                                || r == &ReportResultCodes::null() =>
                            {
                                if is_allowed_failure(&test_report) {
                                    warn!(
                                        "TestScript '{testscript_name}' passed but is listed with --allow-failure; remove it (TestReport id: {})",
                                        test_report.id.as_deref().unwrap_or("<none>")
                                    );
                                }
                            }
                            r if r == &ReportResultCodes::fail()
                                && is_allowed_failure(&test_report) =>
                            {
                                warn!(
                                    "TestScript '{testscript_name}' FAILED as expected (--allow-failure; file: {testscript_file}, TestReport id: {})",
                                    test_report.id.as_deref().unwrap_or("<none>")
                                );
                                mark_expected_failure(&mut test_report);
                            }
                            r if r == &ReportResultCodes::fail() => {
                                status_code = 1;
                                error!(
                                    "TestScript '{testscript_name}' FAILED (file: {testscript_file}, TestReport id: {})",
                                    test_report.id.as_deref().unwrap_or("<none>")
                                );
                            }
                            _ => status_code = 1,
                        }

                        testreport_entries.push(BundleEntry {
                            request: Some(BundleEntryRequest {
                                method: HttpVerb::put(),
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
                        status_code = 1;
                        error!(
                            "TestScript '{testscript_name}' ERRORED (file: {testscript_file}): {e:?}"
                        );
                    }
                }
            }

            let testreport_bundle = Bundle {
                type_: BundleType::transaction(),
                entry: Some(testreport_entries),
                ..Default::default()
            };

            if let Some(output) = output {
                tokio::fs::write(
                    output,
                    serde_json::to_string(&testreport_bundle).map_err(|e| {
                        OperationOutcomeError::fatal(
                            IssueType::exception(),
                            format!("Failed to serialize TestReport bundle: {}", e),
                        )
                    })?,
                )
                .await
                .expect("Failed to write TestReport bundle to file");
            } else {
                println!(
                    "{}",
                    serde_json::to_string(&testreport_bundle).map_err(|e| {
                        OperationOutcomeError::fatal(
                            IssueType::exception(),
                            format!("Failed to serialize TestReport bundle: {}", e),
                        )
                    })?
                );
            }

            if status_code != 0 {
                Err(OperationOutcomeError::fatal(
                    IssueType::exception(),
                    "One or more TestScripts failed".to_string(),
                ))
            } else {
                Ok(())
            }
        }
    }
}
