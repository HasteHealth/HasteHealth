use oxidized_fhir_generated_ops::generated::{
    CodeSystemLookup, ValueSetExpand, ValueSetValidateCode,
};
use oxidized_fhir_model::r4::generated::resources::{Resource, ResourceType};
use oxidized_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use std::pin::Pin;

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

pub trait CanonicalResolver {
    fn resolve(
        &self,
        resource_type: ResourceType,
        id: String,
    ) -> Pin<Box<dyn Future<Output = Result<Resource, OperationOutcomeError>> + Send + Sync>>;
}

pub trait FHIRTerminology {
    fn expand(
        &self,
        input: ValueSetExpand::Input,
    ) -> impl Future<Output = Result<ValueSetExpand::Output, OperationOutcomeError>> + Send;
    fn validate(
        &self,

        input: ValueSetValidateCode::Input,
    ) -> impl Future<Output = Result<ValueSetValidateCode::Output, OperationOutcomeError>> + Send;
    fn lookup(
        &self,
        input: CodeSystemLookup::Input,
    ) -> impl Future<Output = Result<CodeSystemLookup::Output, OperationOutcomeError>> + Send;
}
