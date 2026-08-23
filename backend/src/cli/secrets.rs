//! Storage for CLI secret material (OIDC client secrets, cached OAuth tokens).
//!
//! Kept in its own file, separate from [`crate::cli::config`], so that secrets never show
//! up when a user inspects their config (e.g. `haste-health config show-profile`) and so
//! the two files can be handled differently (e.g. excluded from dotfile backups/sync).

use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

/// Cached OAuth tokens for a profile using the `authorization_code` flow, populated by
/// `haste-health login`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct StoredTokens {
    pub(crate) access_token: String,
    pub(crate) refresh_token: Option<String>,
    pub(crate) id_token: Option<String>,
    /// Unix timestamp (seconds) the access token expires at.
    pub(crate) expires_at: i64,
}

/// Secret material for a single profile.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub(crate) struct ProfileSecrets {
    /// OIDC client secret, set for `client-credentials` profiles.
    #[serde(default)]
    pub(crate) client_secret: Option<String>,
    /// Cached tokens, set once `login` succeeds for `authorization-code` profiles.
    #[serde(default)]
    pub(crate) tokens: Option<StoredTokens>,
}

/// All CLI secrets, keyed by profile name. Persisted separately from `CliConfiguration`.
#[derive(Serialize, Deserialize, Debug, Default)]
pub(crate) struct CliSecrets {
    #[serde(default)]
    pub(crate) profiles: HashMap<String, ProfileSecrets>,
}

impl CliSecrets {
    pub(crate) fn profile(&self, name: &str) -> Option<&ProfileSecrets> {
        self.profiles.get(name)
    }

    pub(crate) fn profile_mut(&mut self, name: &str) -> &mut ProfileSecrets {
        self.profiles.entry(name.to_string()).or_default()
    }

    pub(crate) fn remove_profile(&mut self, name: &str) {
        self.profiles.remove(name);
    }
}

fn read_existing_secrets(location: &PathBuf) -> Result<CliSecrets, OperationOutcomeError> {
    let secrets_str = std::fs::read_to_string(location).map_err(|_| {
        OperationOutcomeError::error(
            IssueType::exception(),
            format!(
                "Failed to read secrets file at location '{}'",
                location.to_string_lossy()
            ),
        )
    })?;

    toml::from_str::<CliSecrets>(&secrets_str).map_err(|_| {
        OperationOutcomeError::error(
            IssueType::exception(),
            format!(
                "Failed to parse secrets file at location '{}'",
                location.to_string_lossy()
            ),
        )
    })
}

/// Loads the secrets file, creating an empty one on disk if it doesn't exist yet.
pub(crate) fn load_secrets(location: &PathBuf) -> CliSecrets {
    if let Ok(secrets) = read_existing_secrets(location) {
        return secrets;
    }

    let secrets = CliSecrets::default();
    write_secrets(location, &secrets).expect("Failed to write default secrets file");
    secrets
}

pub(crate) fn write_secrets(
    location: &PathBuf,
    secrets: &CliSecrets,
) -> Result<(), OperationOutcomeError> {
    std::fs::write(location, toml::to_string(secrets).unwrap()).map_err(|_| {
        OperationOutcomeError::error(
            IssueType::exception(),
            format!(
                "Failed to write secrets file at location '{}'",
                location.to_string_lossy()
            ),
        )
    })
}
