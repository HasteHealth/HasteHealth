use crate::environment::EnvironmentConfig;
use haste_fhir_operation_error::OperationOutcomeError;
use std::sync::Arc;

mod environment;

pub trait Config<Key: Into<String>>: Send + Sync {
    /// Gets the value of an environment configuration variable.
    ///
    /// # Errors
    ///
    /// Returns [`OperationOutcomeError`] if the environment variable is not set
    /// or cannot be read.
    fn get(&self, name: Key) -> Result<String, OperationOutcomeError>;
    /// Sets the value of an environment configuration variable.
    ///
    /// # Errors
    ///
    /// Returns [`OperationOutcomeError`] if setting the variable fails.
    fn set(&self, name: Key, value: String) -> Result<(), OperationOutcomeError>;
}

#[derive(Clone, Copy)]
pub enum ConfigType {
    Environment,
}

impl From<&str> for ConfigType {
    fn from(value: &str) -> Self {
        match value {
            "environment" => ConfigType::Environment,
            _ => panic!("Unknown config type"),
        }
    }
}

#[must_use]
pub fn get_config<Key: Into<String>>(config_type: ConfigType) -> Arc<dyn Config<Key>> {
    match config_type {
        ConfigType::Environment => Arc::new(EnvironmentConfig::new(&[".env", ".env.development"])),
    }
}
