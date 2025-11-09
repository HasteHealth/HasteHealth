use std::path::PathBuf;

use clap::Subcommand;
use oxidized_fhir_model::r4::generated::terminology::IssueType;
use oxidized_fhir_operation_error::OperationOutcomeError;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CLIConfiguration {
    active_profile: Option<String>,
    profiles: Vec<Profile>,
}

impl Default for CLIConfiguration {
    fn default() -> Self {
        CLIConfiguration {
            active_profile: None,
            profiles: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Profile {
    name: String,
    r4_url: String,
    oidc_discovery_uri: String,
    auth: ProfileAuth,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ProfileAuth {
    ClientCredentails {
        client_id: String,
        client_secret: String,
    },
    Public {},
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    Show {
        #[arg(short, long)]
        json: bool,
    },
}

pub fn load_config(location: &PathBuf) -> Result<CLIConfiguration, OperationOutcomeError> {
    let config: Result<CLIConfiguration, OperationOutcomeError> = {
        let config_str = std::fs::read_to_string(location).map_err(|_| {
            OperationOutcomeError::error(
                IssueType::Exception(None),
                format!(
                    "Failed to read config file at location '{}'",
                    location.to_string_lossy()
                ),
            )
        })?;
        let config = toml::from_str::<CLIConfiguration>(&config_str).map_err(|_| {
            OperationOutcomeError::error(
                IssueType::Exception(None),
                format!(
                    "Failed to parse config file at location '{}'",
                    location.to_string_lossy()
                ),
            )
        })?;

        Ok(config)
    };

    if let Ok(config) = config {
        Ok(config)
    } else {
        let config = CLIConfiguration::default();

        std::fs::write(location, toml::to_string(&config).unwrap()).map_err(|_| {
            OperationOutcomeError::error(
                IssueType::Exception(None),
                format!(
                    "Failed to write default config file at location '{}'",
                    location.to_string_lossy()
                ),
            )
        })?;

        Ok(config)
    }
}
