use std::{error::Error, fmt::Display};

use fhir_model::r4::types::OperationOutcome;

#[derive(Debug)]
pub struct OperationError {
    _source: Box<dyn Error + Send + Sync>,
    outcome: OperationOutcome,
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
        Some(&*self._source)
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
