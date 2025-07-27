use crate::config::Config;
use fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};

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

impl Config for EnvironmentConfig {
    fn get(&self, name: &str) -> Result<String, OperationOutcomeError> {
        let k = std::env::var(name).map_err(EnvironmentConfigError::from)?;
        Ok(k)
    }
}
