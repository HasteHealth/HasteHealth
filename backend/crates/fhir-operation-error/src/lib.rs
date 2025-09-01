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

pub enum OperationOutcomeCodes {
    Invalid,
    Structure,
    Required,
    Value,
    Invariant,
    Security,
    Login,
    Unknown,
    Expired,
    Forbidden,
    Suppressed,
    Processing,
    NotSupported,
    Duplicate,
    MultipleMatches,
    NotFound,
    Deleted,
    TooLong,
    CodeInvalid,
    Extension,
    TooCostly,
    BusinessRule,
    Conflict,
    Transient,
    LockError,
    NoStore,
    Exception,
    Timeout,
    Incomplete,
    Throttled,
    Informational,
}

impl Into<String> for OperationOutcomeCodes {
    fn into(self) -> String {
        match self {
            OperationOutcomeCodes::Invalid => "invalid".to_string(),
            OperationOutcomeCodes::Structure => "structure".to_string(),
            OperationOutcomeCodes::Required => "required".to_string(),
            OperationOutcomeCodes::Value => "value".to_string(),
            OperationOutcomeCodes::Invariant => "invariant".to_string(),
            OperationOutcomeCodes::Security => "security".to_string(),
            OperationOutcomeCodes::Login => "login".to_string(),
            OperationOutcomeCodes::Unknown => "unknown".to_string(),
            OperationOutcomeCodes::Expired => "expired".to_string(),
            OperationOutcomeCodes::Forbidden => "forbidden".to_string(),
            OperationOutcomeCodes::Suppressed => "suppressed".to_string(),
            OperationOutcomeCodes::Processing => "processing".to_string(),
            OperationOutcomeCodes::NotSupported => "not-supported".to_string(),
            OperationOutcomeCodes::Duplicate => "duplicate".to_string(),
            OperationOutcomeCodes::MultipleMatches => "multiple-matches".to_string(),
            OperationOutcomeCodes::NotFound => "not-found".to_string(),
            OperationOutcomeCodes::Deleted => "deleted".to_string(),
            OperationOutcomeCodes::TooLong => "too-long".to_string(),
            OperationOutcomeCodes::CodeInvalid => "code-invalid".to_string(),
            OperationOutcomeCodes::Extension => "extension".to_string(),
            OperationOutcomeCodes::TooCostly => "too-costly".to_string(),
            OperationOutcomeCodes::BusinessRule => "business-rule".to_string(),
            OperationOutcomeCodes::Conflict => "conflict".to_string(),
            OperationOutcomeCodes::Transient => "transient".to_string(),
            OperationOutcomeCodes::LockError => "lock-error".to_string(),
            OperationOutcomeCodes::NoStore => "no-store".to_string(),
            OperationOutcomeCodes::Exception => "exception".to_string(),
            OperationOutcomeCodes::Timeout => "timeout".to_string(),
            OperationOutcomeCodes::Incomplete => "incomplete".to_string(),
            OperationOutcomeCodes::Throttled => "throttled".to_string(),
            OperationOutcomeCodes::Informational => "informational".to_string(),
        }
    }
}

impl TryFrom<String> for OperationOutcomeCodes {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "invalid" => Ok(OperationOutcomeCodes::Invalid),
            "structure" => Ok(OperationOutcomeCodes::Structure),
            "required" => Ok(OperationOutcomeCodes::Required),
            "value" => Ok(OperationOutcomeCodes::Value),
            "invariant" => Ok(OperationOutcomeCodes::Invariant),
            "security" => Ok(OperationOutcomeCodes::Security),
            "login" => Ok(OperationOutcomeCodes::Login),
            "unknown" => Ok(OperationOutcomeCodes::Unknown),
            "expired" => Ok(OperationOutcomeCodes::Expired),
            "forbidden" => Ok(OperationOutcomeCodes::Forbidden),
            "suppressed" => Ok(OperationOutcomeCodes::Suppressed),
            "processing" => Ok(OperationOutcomeCodes::Processing),
            "not-supported" => Ok(OperationOutcomeCodes::NotSupported),
            "duplicate" => Ok(OperationOutcomeCodes::Duplicate),
            "multiple-matches" => Ok(OperationOutcomeCodes::MultipleMatches),
            "not-found" => Ok(OperationOutcomeCodes::NotFound),
            "deleted" => Ok(OperationOutcomeCodes::Deleted),
            "too-long" => Ok(OperationOutcomeCodes::TooLong),
            "code-invalid" => Ok(OperationOutcomeCodes::CodeInvalid),
            "extension" => Ok(OperationOutcomeCodes::Extension),
            "too-costly" => Ok(OperationOutcomeCodes::TooCostly),
            "business-rule" => Ok(OperationOutcomeCodes::BusinessRule),
            "conflict" => Ok(OperationOutcomeCodes::Conflict),
            "transient" => Ok(OperationOutcomeCodes::Transient),
            "lock-error" => Ok(OperationOutcomeCodes::LockError),
            "no-store" => Ok(OperationOutcomeCodes::NoStore),
            "exception" => Ok(OperationOutcomeCodes::Exception),
            "timeout" => Ok(OperationOutcomeCodes::Timeout),
            "incomplete" => Ok(OperationOutcomeCodes::Incomplete),
            "throttled" => Ok(OperationOutcomeCodes::Throttled),
            "informational" => Ok(OperationOutcomeCodes::Informational),
            _ => Err(format!("Unknown operation outcome code: {}", value)),
        }
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

    pub fn fatal(code: OperationOutcomeCodes, diagnostic: String) -> Self {
        OperationOutcomeError::new(
            None,
            create_operation_outcome("fatal".to_string(), code.into(), diagnostic),
        )
    }
    pub fn error(code: OperationOutcomeCodes, diagnostic: String) -> Self {
        OperationOutcomeError::new(
            None,
            create_operation_outcome("error".to_string(), code.into(), diagnostic),
        )
    }
    pub fn warning(code: OperationOutcomeCodes, diagnostic: String) -> Self {
        OperationOutcomeError::new(
            None,
            create_operation_outcome("warning".to_string(), code.into(), diagnostic),
        )
    }
    pub fn information(code: OperationOutcomeCodes, diagnostic: String) -> Self {
        OperationOutcomeError::new(
            None,
            create_operation_outcome("information".to_string(), code.into(), diagnostic),
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
