use crate::Config;
use oxidized_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};

pub struct EnvironmentConfig();

#[derive(OperationOutcomeError, Debug)]
pub enum EnvironmentConfigError {
    #[error(code = "invalid", diagnostic = "Invalid environment!")]
    FailedToLoadEnvironment(#[from] dotenvy::Error),
    #[error(code = "invalid", diagnostic = "Environment is misconfigured.")]
    EnvironmentVariableNotSet(#[from] std::env::VarError),
}

impl EnvironmentConfig {
    pub fn new() -> Result<Self, OperationOutcomeError> {
        dotenvy::dotenv().map_err(EnvironmentConfigError::from)?;
        Ok(EnvironmentConfig())
    }
}

impl<Key: Into<String>> Config<Key> for EnvironmentConfig {
    fn get(&self, key: Key) -> Result<String, OperationOutcomeError> {
        let k = std::env::var(key.into()).map_err(EnvironmentConfigError::from)?;
        Ok(k)
    }
    fn set(&self, key: Key, value: String) -> Result<(), OperationOutcomeError> {
        unsafe {
            std::env::set_var(key.into(), value);
        }
        Ok(())
    }
}
