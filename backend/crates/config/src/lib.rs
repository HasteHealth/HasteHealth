use crate::environment::EnvironmentConfig;
use oxidized_fhir_operation_error::OperationOutcomeError;
use std::sync::Arc;

mod environment;

pub trait Config: Send + Sync {
    fn get(&self, name: &str) -> Result<String, OperationOutcomeError>;
    fn set(&self, name: &str, value: String) -> Result<(), OperationOutcomeError>;
}

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

pub fn get_config(config_type: ConfigType) -> Arc<dyn Config> {
    match config_type {
        ConfigType::Environment => Arc::new(EnvironmentConfig::new().unwrap()),
    }
}
