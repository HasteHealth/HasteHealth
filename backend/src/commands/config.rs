use clap::Subcommand;
use dialoguer::Select;
use oxidized_fhir_model::r4::generated::terminology::IssueType;
use oxidized_fhir_operation_error::OperationOutcomeError;
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::CONFIG_LOCATION;

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
    ShowProfile,
    CreateProfile {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        r4_url: String,
        #[arg(short, long)]
        oidc_discovery_uri: String,
        #[arg(long)]
        client_id: String,
        #[arg(long)]
        client_secret: String,
    },
    DeleteProfile {
        #[arg(short, long)]
        name: String,
    },
    SetActiveProfile,
}

fn read_existing_config(location: &PathBuf) -> Result<CLIConfiguration, OperationOutcomeError> {
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
}

pub fn load_config(location: &PathBuf) -> CLIConfiguration {
    let config: Result<CLIConfiguration, OperationOutcomeError> = read_existing_config(location);

    if let Ok(config) = config {
        config
    } else {
        let config = CLIConfiguration::default();

        std::fs::write(location, toml::to_string(&config).unwrap())
            .map_err(|_| {
                OperationOutcomeError::error(
                    IssueType::Exception(None),
                    format!(
                        "Failed to write default config file at location '{}'",
                        location.to_string_lossy()
                    ),
                )
            })
            .expect("Failed to write default config file");

        config
    }
}

pub async fn config(
    config: &Arc<Mutex<CLIConfiguration>>,
    command: &ConfigCommands,
) -> Result<(), OperationOutcomeError> {
    match command {
        ConfigCommands::ShowProfile => {
            let config = config.lock().unwrap();
            if let Some(active_profile_id) = config.active_profile.as_ref()
                && let Some(active_profile) = config
                    .profiles
                    .iter()
                    .find(|p| &p.name == active_profile_id)
            {
                println!("{:#?}", active_profile);
            } else {
                println!("No active profile set.");
            }

            Ok(())
        }
        ConfigCommands::CreateProfile {
            name,
            r4_url,
            oidc_discovery_uri,
            client_id,
            client_secret,
        } => {
            let mut config = config.lock().unwrap();
            if config.profiles.iter().any(|profile| profile.name == *name) {
                return Err(OperationOutcomeError::error(
                    IssueType::Exception(None),
                    format!("Profile with name '{}' already exists", name),
                ));
            }

            let profile = Profile {
                name: name.clone(),
                r4_url: r4_url.clone(),
                oidc_discovery_uri: oidc_discovery_uri.clone(),
                auth: ProfileAuth::ClientCredentails {
                    client_id: client_id.clone(),
                    client_secret: client_secret.clone(),
                },
            };

            config.profiles.push(profile);
            config.active_profile = Some(name.clone());

            std::fs::write(&*CONFIG_LOCATION, toml::to_string(&*config).unwrap()).map_err(
                |_| {
                    OperationOutcomeError::error(
                        IssueType::Exception(None),
                        format!(
                            "Failed to write config file at location '{}'",
                            CONFIG_LOCATION.to_string_lossy()
                        ),
                    )
                },
            )?;

            Ok(())
        }
        ConfigCommands::DeleteProfile { name } => {
            let mut config = config.lock().unwrap();
            config.profiles.retain(|profile| profile.name != *name);

            std::fs::write(&*CONFIG_LOCATION, toml::to_string(&*config).unwrap()).map_err(
                |_| {
                    OperationOutcomeError::error(
                        IssueType::Exception(None),
                        format!(
                            "Failed to write config file at location '{}'",
                            CONFIG_LOCATION.to_string_lossy()
                        ),
                    )
                },
            )?;

            Ok(())
        }
        ConfigCommands::SetActiveProfile => {
            let mut config = config.lock().unwrap();
            let user_profile_names = config
                .profiles
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>();

            if user_profile_names.is_empty() {
                return Err(OperationOutcomeError::error(
                    IssueType::Exception(None),
                    "No profiles available to set as active.".to_string(),
                ));
            }

            let active_profile_index = config
                .active_profile
                .as_ref()
                .and_then(|active_name| {
                    user_profile_names
                        .iter()
                        .position(|&name| name == active_name)
                })
                .unwrap_or(0);

            let selection = Select::new()
                .with_prompt("Choose a profile to set as active.")
                .items(&user_profile_names)
                .default(active_profile_index)
                .interact()
                .unwrap();

            let name = user_profile_names[selection];

            if !config.profiles.iter().any(|profile| profile.name == *name) {
                return Err(OperationOutcomeError::error(
                    IssueType::Exception(None),
                    format!("Profile with name '{}' does not exist", name),
                ));
            }

            config.active_profile = Some(name.to_string());

            std::fs::write(&*CONFIG_LOCATION, toml::to_string(&*config).unwrap()).map_err(
                |_| {
                    OperationOutcomeError::error(
                        IssueType::Exception(None),
                        format!(
                            "Failed to write config file at location '{}'",
                            CONFIG_LOCATION.to_string_lossy()
                        ),
                    )
                },
            )?;
            Ok(())
        }
    }
}
