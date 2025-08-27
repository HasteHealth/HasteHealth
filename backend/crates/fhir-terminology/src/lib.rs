use oxidized_fhir_generated_ops::{CodeSystemLookup, ValueSetExpand, ValueSetValidateCode};
use oxidized_fhir_operation_error::derive::OperationOutcomeError;
use oxidized_repository::types::{ProjectId, TenantId};

pub mod client;

#[derive(OperationOutcomeError, Debug)]
pub enum TerminologyError {
    #[error(code = "processing", diagnostic = "Failed to expand value set")]
    ExpansionError,
    #[error(code = "processing", diagnostic = "Failed to validate code")]
    ValidationError,
    #[error(code = "processing", diagnostic = "Failed to lookup code system")]
    LookupError,
}

pub trait FHIRTerminology<CTX> {
    fn expand(
        &self,
        ctx: CTX,
        input: &ValueSetExpand::Input,
    ) -> Result<ValueSetExpand::Output, TerminologyError>;
    fn validate(
        &self,
        ctx: CTX,
        input: &ValueSetValidateCode::Input,
    ) -> Result<ValueSetValidateCode::Output, TerminologyError>;
    fn lookup(
        &self,
        ctx: CTX,
        input: &CodeSystemLookup::Input,
    ) -> Result<CodeSystemLookup::Output, TerminologyError>;
}
