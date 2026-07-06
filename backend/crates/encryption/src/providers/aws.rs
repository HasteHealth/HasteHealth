use haste_fhir_operation_error::OperationOutcomeError;
use std::{future::Future, pin::Pin};

use crate::{
    error::EncryptionError,
    traits::{Secret, SecretsProvider},
};

/// Retrieves secrets from AWS Secrets Manager.
///
/// Not yet implemented: wiring this up requires adding the `aws-config` /
/// `aws-sdk-secretsmanager` crates, which is deferred until this provider
/// is actually needed.
pub struct AwsSecretsManagerProvider {
    _region: String,
}

impl AwsSecretsManagerProvider {
    pub fn new(region: String) -> Self {
        Self { _region: region }
    }
}

impl SecretsProvider for AwsSecretsManagerProvider {
    fn get_secret<'a>(
        &'a self,
        _name: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Secret, OperationOutcomeError>> + Send + 'a>> {
        Box::pin(async move { Err(EncryptionError::ProviderNotImplemented("aws".to_string()))? })
    }
}
