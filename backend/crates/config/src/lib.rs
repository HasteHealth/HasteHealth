use crate::environment::EnvironmentConfig;
use oxidized_fhir_operation_error::OperationOutcomeError;

mod environment;

pub trait Config: Send + Sync {
    fn get(&self, name: &str) -> Result<String, OperationOutcomeError>;
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

pub fn get_config(config_type: ConfigType) -> Box<dyn Config> {
    match config_type {
        ConfigType::Environment => Box::new(EnvironmentConfig::new().unwrap()),
    }
}
