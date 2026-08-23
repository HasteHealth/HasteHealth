//! Manage named server connection profiles (`~/.haste_health/config.toml`).
//!
//! Secret material (client secrets, cached OAuth tokens) is *not* stored here — see
//! [`crate::commands::secrets`] — so this file is safe to inspect, back up, or share.

use crate::{CLIState, CONFIG_LOCATION, SECRETS_LOCATION, secrets};
use clap::{Subcommand, ValueEnum};
use dialoguer::{Confirm, Select};
use dialoguer::{Input, Password, theme::ColorfulTheme};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};
use tokio::sync::Mutex;

#[derive(Serialize, Deserialize, Debug, Default)]
pub(crate) struct CLIConfiguration {
    pub active_profile: Option<String>,
    pub profiles: Vec<Profile>,
}

impl CLIConfiguration {
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

/// How the CLI authenticates for a newly created profile.
#[derive(Clone, Debug, ValueEnum, PartialEq, Eq)]
pub(crate) enum AuthModeChoice {
    /// A confidential (server-to-server) client authenticated with a client secret.
    ClientCredentials,
    /// A public client a human logs into via the browser (authorization_code + PKCE).
    /// Use `haste-health login` afterwards to obtain tokens.
    AuthorizationCode,
}

/// Manage named server connection profiles.
#[derive(Subcommand, Debug)]
pub(crate) enum ConfigCommands {
    /// Print the currently active profile (never includes secrets or tokens).
    ShowProfile,
    /// Create a new profile and set it as active. Prompts interactively for any option
    /// not passed on the command line.
    CreateProfile {
        /// Name to identify this profile by.
        #[arg(short, long)]
        name: Option<String>,
        /// Base URL of the FHIR R4 server.
        #[arg(short, long)]
        r4_url: Option<String>,
        /// OIDC discovery (`.well-known/openid-configuration`) URI.
        #[arg(short, long)]
        discovery_uri: Option<String>,
        /// How the CLI should authenticate as this profile.
        #[arg(long, value_enum)]
        auth_mode: Option<AuthModeChoice>,
        /// OIDC client ID.
        #[arg(short, long)]
        id: Option<String>,
        /// Client secret. Required for --auth-mode client-credentials, ignored otherwise.
        /// Stored in the secrets file, not the profile itself.
        #[arg(short, long)]
        secret: Option<String>,
        /// Loopback redirect URI for --auth-mode authorization-code (must be registered on the server client).
        #[arg(long)]
        redirect_uri: Option<String>,
        /// OAuth scope to request for --auth-mode authorization-code.
        #[arg(long)]
        scope: Option<String>,
    },
    /// Delete a profile and its stored secrets.
    DeleteProfile {
        /// Name of the profile to delete.
        #[arg(short, long)]
        name: Option<String>,
        /// Skip the interactive confirmation prompt.
        #[arg(short, long)]
        confirm: Option<bool>,
    },
    /// Change which profile is used by default.
    SetActiveProfile {
        /// Name of the profile to activate.
        #[arg(short, long)]
        name: Option<String>,
    },
}

fn read_existing_config(location: &PathBuf) -> Result<CLIConfiguration, OperationOutcomeError> {
    let config_str = std::fs::read_to_string(location).map_err(|_| {
        OperationOutcomeError::error(
            IssueType::exception(),
            format!(
                "Failed to read config file at location '{}'",
                location.to_string_lossy()
            ),
        )
    })?;

    let config = toml::from_str::<CLIConfiguration>(&config_str).map_err(|_| {
        OperationOutcomeError::error(
            IssueType::exception(),
            format!(
                "Failed to parse config file at location '{}'",
                location.to_string_lossy()
            ),
        )
    })?;

    Ok(config)
}

pub(crate) fn load_config(location: &PathBuf) -> CLIConfiguration {
    let config: Result<CLIConfiguration, OperationOutcomeError> = read_existing_config(location);

    if let Ok(config) = config {
        config
    } else {
        let config = CLIConfiguration::default();

        std::fs::write(location, toml::to_string(&config).unwrap())
            .map_err(|_| {
                OperationOutcomeError::error(
                    IssueType::exception(),
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

fn write_config(config: &CLIConfiguration) -> Result<(), OperationOutcomeError> {
    std::fs::write(&*CONFIG_LOCATION, toml::to_string(config).unwrap()).map_err(|_| {
        OperationOutcomeError::error(
            IssueType::exception(),
            format!(
                "Failed to write config file at location '{}'",
                CONFIG_LOCATION.to_string_lossy()
            ),
        )
    })
}

pub(crate) async fn config(
    state: &Arc<Mutex<CLIState>>,
    command: &ConfigCommands,
) -> Result<(), OperationOutcomeError> {
    match command {
        ConfigCommands::ShowProfile => {
            let state = state.lock().await;
            if let Some(active_profile) = state.config.current_profile() {
                println!("{:#?}", active_profile);
            } else {
                println!("No active profile set.");
            }

            Ok(())
        }
        ConfigCommands::CreateProfile {
            name,
            r4_url,
            discovery_uri,
            auth_mode,
            id,
            secret,
            redirect_uri,
            scope,
        } => {
            let name: String = if let Some(name) = name {
                name.clone()
            } else {
                Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Profile Name")
                    .interact_text()
                    .unwrap()
            };

            let r4_url: String = if let Some(r4_url) = r4_url {
                r4_url.clone()
            } else {
                Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("FHIR R4 Server URL")
                    .interact_text()
                    .unwrap()
            };

            let oidc_discovery_uri: String = if let Some(discovery_uri) = discovery_uri {
                discovery_uri.clone()
            } else {
                Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("OIDC Discovery URI")
                    .interact_text()
                    .unwrap()
            };

            let client_id: String = if let Some(id) = id {
                id.clone()
            } else {
                Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("OIDC Client ID")
                    .interact_text()
                    .unwrap()
            };

            let auth_mode: AuthModeChoice = match auth_mode {
                Some(mode) => mode.clone(),
                None => {
                    let options = ["Authorization Code (browser login)", "Client Credentials"];
                    let selection = Select::with_theme(&ColorfulTheme::default())
                        .with_prompt("Auth Mode")
                        .items(&options)
                        .default(0)
                        .interact()
                        .unwrap();

                    match selection {
                        1 => AuthModeChoice::ClientCredentials,
                        _ => AuthModeChoice::AuthorizationCode,
                    }
                }
            };

            let (auth, client_secret) = match auth_mode {
                AuthModeChoice::ClientCredentials => {
                    let client_secret: String = if let Some(secret) = secret {
                        secret.clone()
                    } else {
                        Password::with_theme(&ColorfulTheme::default())
                            .with_prompt("OIDC Client Secret")
                            .interact()
                            .unwrap()
                    };

                    (
                        ProfileAuth::ClientCredentails {
                            client_id: client_id.clone(),
                        },
                        Some(client_secret),
                    )
                }
                AuthModeChoice::AuthorizationCode => {
                    let redirect_uri: String = if let Some(redirect_uri) = redirect_uri {
                        redirect_uri.clone()
                    } else {
                        Input::with_theme(&ColorfulTheme::default())
                            .with_prompt("Loopback Redirect URI")
                            .default("http://127.0.0.1:8976/callback".to_string())
                            .interact_text()
                            .unwrap()
                    };

                    let scope: String = if let Some(scope) = scope {
                        scope.clone()
                    } else {
                        Input::with_theme(&ColorfulTheme::default())
                            .with_prompt("OAuth Scope")
                            .default("openid profile fhirUser offline_access user/*.*".to_string())
                            .interact_text()
                            .unwrap()
                    };

                    (
                        ProfileAuth::AuthorizationCode {
                            client_id: client_id.clone(),
                            redirect_uri,
                            scope,
                        },
                        None,
                    )
                }
            };

            let mut state = state.lock().await;
            if state
                .config
                .profiles
                .iter()
                .any(|profile| profile.name == *name)
            {
                return Err(OperationOutcomeError::error(
                    IssueType::exception(),
                    format!("Profile with name '{}' already exists", name),
                ));
            }

            let profile = Profile {
                name: name.clone(),
                r4_url: r4_url.clone(),
                oidc_discovery_uri: oidc_discovery_uri.clone(),
                auth,
            };

            state.config.profiles.push(profile);
            state.config.active_profile = Some(name.clone());

            if let Some(client_secret) = client_secret {
                state.secrets.profile_mut(&name).client_secret = Some(client_secret);
                secrets::write_secrets(&SECRETS_LOCATION, &state.secrets)?;
            }

            write_config(&state.config)
        }
        ConfigCommands::DeleteProfile { name, confirm } => {
            let name: String = if let Some(name) = name {
                name.clone()
            } else {
                Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter the profile name you wish to delete")
                    .interact_text()
                    .unwrap()
            };

            let confirmed = if let Some(confirm) = confirm {
                confirm.clone()
            } else {
                Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt(format!(
                        "Are you sure you want to delete the profile '{}'? ",
                        name
                    ))
                    .interact()
                    .unwrap_or(false)
            };

            if !confirmed {
                println!("Profile deletion cancelled.");
                return Ok(());
            }

            let mut state = state.lock().await;
            state
                .config
                .profiles
                .retain(|profile| profile.name != *name);
            state.secrets.remove_profile(&name);

            secrets::write_secrets(&SECRETS_LOCATION, &state.secrets)?;
            write_config(&state.config)
        }
        ConfigCommands::SetActiveProfile { name } => {
            let mut state = state.lock().await;
            let user_profile_names = state
                .config
                .profiles
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>();

            if user_profile_names.is_empty() {
                return Err(OperationOutcomeError::error(
                    IssueType::exception(),
                    "No profiles available to set as active.".to_string(),
                ));
            }

            let active_profile_index = state
                .config
                .active_profile
                .as_ref()
                .and_then(|active_name| {
                    user_profile_names
                        .iter()
                        .position(|&name| name == active_name)
                })
                .unwrap_or(0);

            let name: String = if let Some(name) = name {
                name.clone()
            } else {
                let selection = Select::new()
                    .with_prompt("Choose a profile to set as active.")
                    .items(&user_profile_names)
                    .default(active_profile_index)
                    .interact()
                    .unwrap();
                user_profile_names[selection].to_string()
            };

            if !state
                .config
                .profiles
                .iter()
                .any(|profile| profile.name == name)
            {
                return Err(OperationOutcomeError::error(
                    IssueType::exception(),
                    format!("Profile with name '{}' does not exist", name),
                ));
            }

            state.config.active_profile = Some(name.to_string());

            write_config(&state.config)
        }
    }
}
