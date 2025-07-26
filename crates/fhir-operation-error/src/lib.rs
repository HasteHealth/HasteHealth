use std::{error::Error, fmt::Display};

use fhir_model::r4::types::OperationOutcome;

#[cfg(feature = "derive")]
pub mod derive;

#[derive(Debug)]
pub struct OperationError {
    _source: Option<anyhow::Error>,
    outcome: OperationOutcome,
}

impl OperationError {
    pub fn new(source: Option<anyhow::Error>, outcome: OperationOutcome) -> Self {
        OperationError {
            _source: source,
            outcome,
        }
    }

    pub fn outcome(&self) -> &OperationOutcome {
        &self.outcome
    }

    pub fn push_issue(&mut self, issue: fhir_model::r4::types::OperationOutcomeIssue) {
        self.outcome.issue.push(issue);
    }

    pub fn backtrace(&self) -> Option<&std::backtrace::Backtrace> {
        self._source.as_ref().map(|s| s.backtrace())
    }
}

fn get_issue_diagnostics<'a>(
    issue: &'a fhir_model::r4::types::OperationOutcomeIssue,
) -> Option<&'a str> {
    issue
        .diagnostics
        .as_ref()
        .and_then(|d| d.value.as_ref().map(|v| v.as_str()))
}

impl Display for OperationError {
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

impl Error for OperationError {
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
