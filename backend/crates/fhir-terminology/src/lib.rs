use core::error;
use oxidized_fhir_generated_ops::{CodeSystemLookup, ValueSetExpand, ValueSetValidateCode};
use oxidized_fhir_operation_error::derive::OperationOutcomeError;

#[derive(OperationOutcomeError, Debug)]
pub enum TerminologyError {
    #[error(code = "processing", diagnostic = "Failed to expand value set")]
    ExpansionError,
    #[error(code = "processing", diagnostic = "Failed to validate code")]
    ValidationError,
    #[error(code = "processing", diagnostic = "Failed to lookup code system")]
    LookupError,
}

pub trait FHIRTerminology {
    fn expand(
        &self,
        input: &ValueSetExpand::Input,
    ) -> Result<ValueSetExpand::Output, TerminologyError>;
    fn validate(
        &self,
        input: &ValueSetValidateCode::Input,
    ) -> Result<ValueSetValidateCode::Output, TerminologyError>;
    fn lookup(
        &self,
        input: &CodeSystemLookup::Input,
    ) -> Result<CodeSystemLookup::Output, TerminologyError>;
}
