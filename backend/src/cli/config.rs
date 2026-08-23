//! Schema and on-disk persistence for named server connection profiles
//! (`~/.haste_health/config.toml`). The `config` *command* (create/delete/list profiles)
//! lives in [`crate::commands::config`]; this module only owns the data.
//!
//! Secret material (client secrets, cached OAuth tokens) is never stored here — see
//! [`crate::cli::secrets`] — so this file is safe to inspect, back up, or share.

use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Default)]
pub(crate) struct CliConfiguration {
    pub(crate) active_profile: Option<String>,
    pub(crate) profiles: Vec<Profile>,
}

impl CliConfiguration {
    pub(crate) fn current_profile(&self) -> Option<&Profile> {
        if let Some(active_profile_id) = self.active_profile.as_ref() {
            self.profiles.iter().find(|p| &p.name == active_profile_id)
        } else {
            None
        }
    }
}

/// A named connection to a FHIR server plus how the CLI should authenticate to it.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Profile {
    pub(crate) name: String,
    pub(crate) r4_url: String,
    pub(crate) oidc_discovery_uri: String,
    pub(crate) auth: ProfileAuth,
}

/// How the CLI authenticates for a given profile. Any secret values (client secret,
/// cached tokens) live in the separate secrets file, keyed by profile name.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) enum ProfileAuth {
    /// A confidential (server-to-server) client authenticated with a client secret.
    ClientCredentails { client_id: String },
    /// A public (no secret) OIDC client authenticated by a human via the browser-based
    /// authorization_code + PKCE flow. Run `haste-health login` to obtain tokens.
    AuthorizationCode {
        client_id: String,
        redirect_uri: String,
        scope: String,
    },
    /// No authentication; requests are sent unauthenticated.
    Public {},
}

fn read_existing_config(location: &PathBuf) -> Result<CliConfiguration, OperationOutcomeError> {
    let config_str = std::fs::read_to_string(location).map_err(|_| {
        OperationOutcomeError::error(
            IssueType::exception(),
            format!(
                "Failed to read config file at location '{}'",
                location.to_string_lossy()
            ),
        )
    })?;

    toml::from_str::<CliConfiguration>(&config_str).map_err(|_| {
        OperationOutcomeError::error(
            IssueType::exception(),
            format!(
                "Failed to parse config file at location '{}'",
                location.to_string_lossy()
            ),
        )
    })
}

/// Loads the config file, creating a default one on disk if it doesn't exist yet.
pub(crate) fn load_config(location: &PathBuf) -> CliConfiguration {
    if let Ok(config) = read_existing_config(location) {
        return config;
    }

    let config = CliConfiguration::default();
    write_config(location, &config).expect("Failed to write default config file");
    config
}

pub(crate) fn write_config(
    location: &PathBuf,
    config: &CliConfiguration,
) -> Result<(), OperationOutcomeError> {
    std::fs::write(location, toml::to_string(config).unwrap()).map_err(|_| {
        OperationOutcomeError::error(
            IssueType::exception(),
            format!(
                "Failed to write config file at location '{}'",
                location.to_string_lossy()
            ),
        )
    })
}
