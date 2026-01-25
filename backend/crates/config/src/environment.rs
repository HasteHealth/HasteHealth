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
    pub fn new(config_files: &[&str]) -> Result<Self, OperationOutcomeError> {
        for file in config_files {
            let file_result = dotenvy::from_filename(file).map_err(EnvironmentConfigError::from);
            if let Err(e) = file_result {
                tracing::warn!("Failed to load environment file '{}' '{:?}'", file, e)
            }
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
