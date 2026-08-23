//! `config` command: manage named server connection profiles.
//!
//! This module only owns the CLI surface (argument parsing, prompts). The persisted
//! schema lives in [`crate::cli::config`]; secrets in [`crate::cli::secrets`].

use crate::cli::{
    config::{CliConfiguration, Profile, ProfileAuth, write_config},
    secrets,
    state::{CONFIG_LOCATION, CliState, SECRETS_LOCATION},
};
use clap::{Subcommand, ValueEnum};
use dialoguer::{Confirm, Select};
use dialoguer::{Input, Password, theme::ColorfulTheme};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use std::sync::Arc;
use tokio::sync::Mutex;

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

fn persist(config: &CliConfiguration) -> Result<(), OperationOutcomeError> {
    write_config(&CONFIG_LOCATION, config)
}

fn select_profile_name(state: &CliState, prompt: &str) -> Result<String, OperationOutcomeError> {
    let profile_names = state
        .config
        .profiles
        .iter()
        .map(|profile| profile.name.as_str())
        .collect::<Vec<_>>();

    if profile_names.is_empty() {
        return Err(OperationOutcomeError::error(
            IssueType::exception(),
            "No profiles available.".to_string(),
        ));
    }

    let active_profile_index = state
        .config
        .active_profile
        .as_ref()
        .and_then(|active_name| profile_names.iter().position(|&name| name == active_name))
        .unwrap_or(0);

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(&profile_names)
        .default(active_profile_index)
        .interact()
        .unwrap();

    Ok(profile_names[selection].to_string())
}

/// Runs the `config` command group.
pub(crate) async fn run(
    state: Arc<Mutex<CliState>>,
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

            persist(&state.config)
        }
        ConfigCommands::DeleteProfile { name, confirm } => {
            let name: String = if let Some(name) = name {
                name.clone()
            } else {
                let state = state.lock().await;
                select_profile_name(&state, "Choose a profile to delete")?
            };

            let confirmed = if let Some(confirm) = confirm {
                *confirm
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
            persist(&state.config)
        }
        ConfigCommands::SetActiveProfile { name } => {
            let mut state = state.lock().await;
            let name: String = if let Some(name) = name {
                name.clone()
            } else {
                select_profile_name(&state, "Choose a profile to set as active")?
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

            persist(&state.config)
        }
    }
}
