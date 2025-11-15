use std::{error::Error, fmt::Display};

use haste_fhir_model::r4::generated::{
    resources::{OperationOutcome, OperationOutcomeIssue},
    terminology::{IssueSeverity, IssueType},
    types::FHIRString,
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
    severity: IssueSeverity,
    code: IssueType,
    diagnostic: String,
) -> OperationOutcome {
    OperationOutcome {
        issue: vec![OperationOutcomeIssue {
            severity: Box::new(severity),
            code: Box::new(code),
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

    pub fn push_issue(
        &mut self,
        issue: haste_fhir_model::r4::generated::resources::OperationOutcomeIssue,
    ) {
        self.outcome.issue.push(issue);
    }

    pub fn backtrace(&self) -> Option<&std::backtrace::Backtrace> {
        self._source.as_ref().map(|s| s.backtrace())
    }

    pub fn fatal(code: IssueType, diagnostic: String) -> Self {
        OperationOutcomeError::new(
            None,
            create_operation_outcome(IssueSeverity::Fatal(None), code, diagnostic),
        )
    }
    pub fn error(code: IssueType, diagnostic: String) -> Self {
        OperationOutcomeError::new(
            None,
            create_operation_outcome(IssueSeverity::Error(None), code, diagnostic),
        )
    }
    pub fn warning(code: IssueType, diagnostic: String) -> Self {
        OperationOutcomeError::new(
            None,
            create_operation_outcome(IssueSeverity::Warning(None), code, diagnostic),
        )
    }
    pub fn information(code: IssueType, diagnostic: String) -> Self {
        OperationOutcomeError::new(
            None,
            create_operation_outcome(IssueSeverity::Information(None), code, diagnostic),
        )
    }
}

fn get_issue_diagnostics<'a>(
    issue: &'a haste_fhir_model::r4::generated::resources::OperationOutcomeIssue,
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
