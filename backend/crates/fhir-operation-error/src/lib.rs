use std::{error::Error, fmt::Display};

use haste_fhir_model::r4::generated::{
    resources::{OperationOutcome, OperationOutcomeIssue},
    terminology::{BoundCode, IssueSeverity, IssueType},
    types::FHIRString,
};

#[cfg(feature = "derive")]
pub mod derive;

#[cfg(feature = "axum")]
pub mod axum;

#[derive(Debug)]
pub struct OperationOutcomeError {
    source: Option<anyhow::Error>,
    outcome: Box<OperationOutcome>,
}

fn create_operation_outcome(
    severity: BoundCode<IssueSeverity>,
    code: BoundCode<IssueType>,
    diagnostic: String,
) -> OperationOutcome {
    OperationOutcome {
        issue: vec![OperationOutcomeIssue {
            severity,
            code,
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
    #[must_use]
    pub fn new(source: Option<anyhow::Error>, outcome: OperationOutcome) -> Self {
        OperationOutcomeError {
            source,
            outcome: Box::new(outcome),
        }
    }

    #[must_use]
    pub fn outcome(&self) -> &OperationOutcome {
        &self.outcome
    }

    pub fn push_issue(
        &mut self,
        issue: haste_fhir_model::r4::generated::resources::OperationOutcomeIssue,
    ) {
        self.outcome.issue.push(issue);
    }

    #[must_use]
    pub fn backtrace(&self) -> Option<&std::backtrace::Backtrace> {
        self.source.as_ref().map(anyhow::Error::backtrace)
    }

    #[must_use]
    pub fn fatal(code: BoundCode<IssueType>, diagnostic: String) -> Self {
        OperationOutcomeError::new(
            None,
            create_operation_outcome(IssueSeverity::FATAL, code, diagnostic),
        )
    }
    #[must_use]
    pub fn error(code: BoundCode<IssueType>, diagnostic: String) -> Self {
        OperationOutcomeError::new(
            None,
            create_operation_outcome(IssueSeverity::ERROR, code, diagnostic),
        )
    }
    #[must_use]
    pub fn warning(code: BoundCode<IssueType>, diagnostic: String) -> Self {
        OperationOutcomeError::new(
            None,
            create_operation_outcome(IssueSeverity::WARNING, code, diagnostic),
        )
    }
    #[must_use]
    pub fn information(code: BoundCode<IssueType>, diagnostic: String) -> Self {
        OperationOutcomeError::new(
            None,
            create_operation_outcome(IssueSeverity::INFORMATION, code, diagnostic),
        )
    }
}

fn get_issue_diagnostics(
    issue: &haste_fhir_model::r4::generated::resources::OperationOutcomeIssue,
) -> Option<&str> {
    issue.diagnostics.as_ref().and_then(|d| d.value.as_deref())
}

impl Display for OperationOutcomeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Operation Error: '{}'",
            self.outcome
                .issue
                .iter()
                .filter_map(get_issue_diagnostics)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl Error for OperationOutcomeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let Some(source) = self.source.as_ref() {
            Some(&**source)
        } else {
            None
        }
    }

    fn description(&self) -> &str {
        self.outcome.issue.first().map_or("Unknown error", |issue| {
            if let Some(diagnostics) = &issue.diagnostics {
                diagnostics
                    .value
                    .as_deref()
                    .map_or("No diagnostics available", |v| v)
            } else {
                "No diagnostics available"
            }
        })
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}
