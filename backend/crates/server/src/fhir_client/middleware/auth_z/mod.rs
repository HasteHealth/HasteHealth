use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;

pub mod access_control;
pub mod patient_scope;
pub mod scope_check;

/// A refusal the caller's authorization explains: HTTP 403.
pub(crate) fn forbidden(message: impl Into<String>) -> OperationOutcomeError {
    OperationOutcomeError::error(IssueType::forbidden(), message.into())
}
