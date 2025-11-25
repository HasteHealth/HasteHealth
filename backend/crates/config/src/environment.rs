use crate::Config;
use haste_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};

pub struct EnvironmentConfig();

#[derive(OperationOutcomeError, Debug)]
pub enum EnvironmentConfigError {
    #[error(code = "invalid", diagnostic = "Invalid environment '{arg0}'!")]
    FailedToLoadEnvironment(#[from] dotenvy::Error),
    #[error(
        code = "invalid",
        diagnostic = "Environment is misconfigured '{arg0}' for key '{arg1}'."
    )]
    EnvironmentVariableNotSet(std::env::VarError, String),
}

impl EnvironmentConfig {
    pub fn new() -> Result<Self, OperationOutcomeError> {
        let result = dotenvy::dotenv().map_err(EnvironmentConfigError::from);

        if let Err(e) = result {
            tracing::warn!(
                "Failed to load .env file: '{:?}' will use strictly environment variables,",
                e
            )
        }

        Ok(EnvironmentConfig())
    }
}

impl<Key: Into<String>> Config<Key> for EnvironmentConfig {
    fn get(&self, key: Key) -> Result<String, OperationOutcomeError> {
        let key_string = key.into();
        let k = std::env::var(&key_string)
            .map_err(|e| EnvironmentConfigError::EnvironmentVariableNotSet(e, key_string))?;
        Ok(k)
    }
    fn set(&self, key: Key, value: String) -> Result<(), OperationOutcomeError> {
        unsafe {
            std::env::set_var(key.into(), value);
        }
        Ok(())
    }
}
