use crate::{
    error::EncryptionError,
    traits::{Secret, SecretsProvider},
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use haste_fhir_operation_error::OperationOutcomeError;
use std::{future::Future, pin::Pin};

/// Reads secrets directly from process environment variables. `name` is
/// used verbatim as the environment variable name, optionally prefixed.
pub struct EnvironmentSecretsProvider {
    prefix: Option<String>,
}

impl EnvironmentSecretsProvider {
    pub fn new(prefix: Option<String>) -> Self {
        Self { prefix }
    }

    fn env_var_name(&self, name: &str) -> String {
        match &self.prefix {
            Some(prefix) => format!("{prefix}{name}"),
            None => name.to_string(),
        }
    }
}

impl SecretsProvider for EnvironmentSecretsProvider {
    fn get_secret<'a>(
        &'a self,
        name: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Secret, OperationOutcomeError>> + Send + 'a>> {
        Box::pin(async move {
            let env_var_name = self.env_var_name(name);

            let value = std::env::var(&env_var_name).map_err(|e| {
                EncryptionError::SecretRetrievalFailed(name.to_string(), e.to_string())
            })?;

            let value = STANDARD.decode(value).map_err(|e| {
                EncryptionError::SecretRetrievalFailed(name.to_string(), e.to_string())
            })?;

            Ok(Secret::new(value))
        })
    }
}
