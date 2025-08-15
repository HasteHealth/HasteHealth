use std::{error::Error, fmt::Display};

use oxidized_fhir_model::r4::types::{
    FHIRCode, FHIRString, OperationOutcome, OperationOutcomeIssue,
};

#[cfg(feature = "derive")]
pub mod derive;

#[cfg(feature = "axum")]
pub mod axum;

#[derive(Debug)]
pub struct OperationOutcomeError {
    _source: Option<anyhow::Error>,
    outcome: OperationOutcome,
}

fn create_operation_outcome(
    severity: String,
    code: String,
    diagnostic: String,
) -> OperationOutcome {
    OperationOutcome {
        issue: vec![OperationOutcomeIssue {
            severity: Box::new(FHIRCode {
                value: Some(severity),
                ..Default::default()
            }),
            code: Box::new(FHIRCode {
                value: Some(code),
                ..Default::default()
            }),
            diagnostics: Some(Box::new(FHIRString {
                value: Some(diagnostic),
                ..Default::default()
            })),
            ..Default::default()
        }],
        ..Default::default()
    }
}

impl OperationOutcomeError {
    pub fn new(source: Option<anyhow::Error>, outcome: OperationOutcome) -> Self {
        OperationOutcomeError {
            _source: source,
            outcome,
        }
    }

    pub fn outcome(&self) -> &OperationOutcome {
        &self.outcome
    }

    pub fn push_issue(&mut self, issue: oxidized_fhir_model::r4::types::OperationOutcomeIssue) {
        self.outcome.issue.push(issue);
    }

    pub fn backtrace(&self) -> Option<&std::backtrace::Backtrace> {
        self._source.as_ref().map(|s| s.backtrace())
    }

    pub fn fatal(code: String, diagnostic: String) -> Self {
        OperationOutcomeError::new(
            None,
            create_operation_outcome("fatal".to_string(), code, diagnostic),
        )
    }
    pub fn error(code: String, diagnostic: String) -> Self {
        OperationOutcomeError::new(
            None,
            create_operation_outcome("error".to_string(), code, diagnostic),
        )
    }
    pub fn warning(code: String, diagnostic: String) -> Self {
        OperationOutcomeError::new(
            None,
            create_operation_outcome("warning".to_string(), code, diagnostic),
        )
    }
    pub fn information(code: String, diagnostic: String) -> Self {
        OperationOutcomeError::new(
            None,
            create_operation_outcome("information".to_string(), code, diagnostic),
        )
    }
}

fn get_issue_diagnostics<'a>(
    issue: &'a oxidized_fhir_model::r4::types::OperationOutcomeIssue,
) -> Option<&'a str> {
    issue
        .diagnostics
        .as_ref()
        .and_then(|d| d.value.as_ref().map(|v| v.as_str()))
}

impl Display for OperationOutcomeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Operation Error: '{}'",
            self.outcome
                .issue
                .iter()
                .map(get_issue_diagnostics)
                .filter_map(|d| d)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl Error for OperationOutcomeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let Some(source) = self._source.as_ref() {
            return Some(&**source);
        } else {
            None
        }
    }

    fn description(&self) -> &str {
        self.outcome.issue.first().map_or("Unknown error", |issue| {
            if let Some(diagnostics) = &issue.diagnostics {
                diagnostics
                    .value
                    .as_ref()
                    .map(|v| v.as_str())
                    .unwrap_or("No diagnostics available")
            } else {
                "No diagnostics available"
            }
        })
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}
