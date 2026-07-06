use haste_fhir_operation_error::OperationOutcomeError;
use std::{future::Future, pin::Pin};

use crate::{
    error::EncryptionError,
    traits::{Secret, SecretsProvider},
};

/// Retrieves secrets from GCP Secret Manager.
///
/// Not yet implemented: there is no established async GCP client in this
/// workspace yet, so wiring this up (REST + service-account/metadata-server
/// auth) is deferred until this provider is actually needed.
pub struct GcpSecretManagerProvider {
    _project_id: String,
}

impl GcpSecretManagerProvider {
    pub fn new(project_id: String) -> Self {
        Self {
            _project_id: project_id,
        }
    }
}

impl SecretsProvider for GcpSecretManagerProvider {
    fn get_secret<'a>(
        &'a self,
        _name: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Secret, OperationOutcomeError>> + Send + 'a>> {
        Box::pin(async move { Err(EncryptionError::ProviderNotImplemented("gcp".to_string()))? })
    }
}
