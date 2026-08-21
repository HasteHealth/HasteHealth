use crate::{CLIState, CONFIG_LOCATION};
use clap::{Subcommand, ValueEnum};
use dialoguer::{Confirm, Select};
use dialoguer::{Input, Password, theme::ColorfulTheme};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};
use tokio::sync::Mutex;

#[derive(Serialize, Deserialize, Debug)]
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

impl Default for CLIConfiguration {
    fn default() -> Self {
        CLIConfiguration {
            active_profile: None,
            profiles: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Profile {
    pub(crate) name: String,
    pub(crate) r4_url: String,
    pub(crate) oidc_discovery_uri: String,
    pub(crate) auth: ProfileAuth,
    /// Cached tokens for `ProfileAuth::AuthorizationCode` profiles, populated by `haste-health login`.
    #[serde(default)]
    pub(crate) tokens: Option<StoredTokens>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) enum ProfileAuth {
    ClientCredentails {
        client_id: String,
        client_secret: String,
    },
    /// A public (no secret) OIDC client authenticated by a human via the browser-based
    /// authorization_code + PKCE flow. Run `haste-health login` to obtain tokens.
    AuthorizationCode {
        client_id: String,
        redirect_uri: String,
        scope: String,
    },
    Public {},
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct StoredTokens {
    pub(crate) access_token: String,
    pub(crate) refresh_token: Option<String>,
    pub(crate) id_token: Option<String>,
    /// Unix timestamp (seconds) the access token expires at.
    pub(crate) expires_at: i64,
}

#[derive(Clone, Debug, ValueEnum, PartialEq, Eq)]
pub(crate) enum AuthModeChoice {
    /// A confidential (server-to-server) client authenticated with a client secret.
    ClientCredentials,
    /// A public client a human logs into via the browser (authorization_code + PKCE).
    /// Use `haste-health login` afterwards to obtain tokens.
    AuthorizationCode,
}

#[derive(Subcommand, Debug)]
pub(crate) enum ConfigCommands {
    ShowProfile,
    CreateProfile {
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short, long)]
        r4_url: Option<String>,
        #[arg(short, long)]
        discovery_uri: Option<String>,
        #[arg(long, value_enum)]
        auth_mode: Option<AuthModeChoice>,
        #[arg(short, long)]
        id: Option<String>,
        /// Client secret. Required for --auth-mode client-credentials, ignored otherwise.
        #[arg(short, long)]
        secret: Option<String>,
        /// Loopback redirect URI for --auth-mode authorization-code (must be registered on the server client).
        #[arg(long)]
        redirect_uri: Option<String>,
        /// OAuth scope to request for --auth-mode authorization-code.
        #[arg(long)]
        scope: Option<String>,
    },
    DeleteProfile {
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short, long)]
        confirm: Option<bool>,
    },
    SetActiveProfile {
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

pub(crate) async fn config(
    state: &Arc<Mutex<CLIState>>,
    command: &ConfigCommands,
) -> Result<(), OperationOutcomeError> {
    match command {
        ConfigCommands::ShowProfile => {
            let state = state.lock().await;
            if let Some(active_profile_id) = state.config.active_profile.as_ref()
                && let Some(active_profile) = state
                    .config
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

            let auth = match auth_mode {
                AuthModeChoice::ClientCredentials => {
                    let client_secret: String = if let Some(secret) = secret {
                        secret.clone()
                    } else {
                        Password::with_theme(&ColorfulTheme::default())
                            .with_prompt("OIDC Client Secret")
                            .interact()
                            .unwrap()
                    };

                    ProfileAuth::ClientCredentails {
                        client_id: client_id.clone(),
                        client_secret,
                    }
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

                    ProfileAuth::AuthorizationCode {
                        client_id: client_id.clone(),
                        redirect_uri,
                        scope,
                    }
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
                tokens: None,
            };

            state.config.profiles.push(profile);
            state.config.active_profile = Some(name.clone());

            std::fs::write(&*CONFIG_LOCATION, toml::to_string(&state.config).unwrap()).map_err(
                |_| {
                    OperationOutcomeError::error(
                        IssueType::exception(),
                        format!(
                            "Failed to write config file at location '{}'",
                            CONFIG_LOCATION.to_string_lossy()
                        ),
                    )
                },
            )?;

            Ok(())
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

            std::fs::write(&*CONFIG_LOCATION, toml::to_string(&state.config).unwrap()).map_err(
                |_| {
                    OperationOutcomeError::error(
                        IssueType::exception(),
                        format!(
                            "Failed to write config file at location '{}'",
                            CONFIG_LOCATION.to_string_lossy()
                        ),
                    )
                },
            )?;

            Ok(())
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

            std::fs::write(&*CONFIG_LOCATION, toml::to_string(&state.config).unwrap()).map_err(
                |_| {
                    OperationOutcomeError::error(
                        IssueType::exception(),
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
