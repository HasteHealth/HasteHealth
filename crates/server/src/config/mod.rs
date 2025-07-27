use fhir_client::request::Operation;
use fhir_operation_error::OperationOutcomeError;

mod environment;

pub trait Config {
    fn get(name: &str) -> Result<String, OperationOutcomeError>;
}
