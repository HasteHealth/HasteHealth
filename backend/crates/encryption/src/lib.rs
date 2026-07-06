mod error;

pub mod encryptor;
pub mod providers;
pub mod traits;

pub use error::EncryptionError;
pub use traits::{Encryptor, Secret, SecretsProvider};

use std::sync::Arc;

pub enum SecretsProviderKind {
    Environment {
        prefix: Option<String>,
    },
    #[cfg(feature = "aws")]
    Aws {
        region: String,
    },
    #[cfg(feature = "gcp")]
    Gcp {
        project_id: String,
    },
}

pub fn get_secrets_provider(kind: SecretsProviderKind) -> Arc<dyn SecretsProvider> {
    match kind {
        SecretsProviderKind::Environment { prefix } => Arc::new(
            providers::environment::EnvironmentSecretsProvider::new(prefix),
        ),
        #[cfg(feature = "aws")]
        SecretsProviderKind::Aws { region } => {
            Arc::new(providers::aws::AwsSecretsManagerProvider::new(region))
        }
        #[cfg(feature = "gcp")]
        SecretsProviderKind::Gcp { project_id } => {
            Arc::new(providers::gcp::GcpSecretManagerProvider::new(project_id))
        }
    }
}
